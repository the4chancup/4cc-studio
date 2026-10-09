# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 3 (Team compiler skeleton) closed 2026-10-02, its cross-family reviews queued (see
"Handover"). Phases 1 and 2 done (Phase 2 closed 2026-09-30).
**Next:** Phase 4 is itemized and its Acceptance section written (step 4.1, 2026-10-03; its
cross-family review (a) is queued). Next:
the next open Phase 4 step (see the list); 4.21 to 4.24 and 4.29
and 4.25 are done; 4.26 is done; 4.19 (Fox referees) is done, 4.19d (pre-Fox) is done, 4.19e (a pre-Fox
referee's shared boots link) is open; 4.27a (the Fox referee marker) is done, 4.27's pre-Fox half landed
with 4.19d, 4.27b is done (4.27 done); 4.9 is done
(collars on Fox; their pre-Fox and cross-format halves are in 4.16 and 4.17); 4.28
(memory accounting), 4.32 (number atlases) and 4.18 (hand auto-split, Fox) are done, and
4.y moved to Phase 8; 4.14's slices are all done (a to d, e1 to e5; its GPT reviews (a) TC-MOD-43 and (b) e3 stay queued); 4.15 done (a1, a2, b: the member's own `face.xml` read, checked and emitted; `mtl_texture_not_found` a Warning on pre-Fox, Fumos's evidence); 4.16 done (a: PES 15-17 kits with the mask template and loose configs; b: `dds_compression`; c: `.model` collars; d: the installed loose kit configs of absent slots patched and re-emitted), with its own checks until 4.31's pre-Fox parity; 4.17 in progress (lead done: rulings and the environment cubemap; a done: a player's FMDL converted for PES 15-17 through the face task, its GPT review (c) queued; b done: a player's `.model` converted for PES 18-21 in the Models task, every converted model checked in its target form, a beaten model read by nothing; c1 done: every conversion loss reported at its catalog severity, the same-engine pre-check measured and deferred to slice g; c2 done: a selected glTF in a player folder refused at planning with `model_gltf_unsupported`, TC-MOD-28; d done: the template environment cubemap emitted for a converted metal material on PES 15-17, its GPT review (d) queued, TC-MOD-36; e done: an FMDL collar converted for PES 15-17 with the stock names, a glTF collar dropped at planning, TC-CMN-09; f1 done: a shared folder's other-format model converted as a player's, a shared folder's glTF dropping its linking players; f2 done: a shared boots or gloves folder's FMDL and an `ingame_face` player's FMDL parts converted by the pre-Fox boots and gloves writer, a conversion finding naming a shared source by its export path; f3a done: the glTF drop's orphans removed, an `env` link the environment map, every `model` context named alike, one sampler shape, `kit_variant_model_fox` Fox-only; f3b done: a Common FMDL converted once in the Common models task on PES 15-17, its link listing the conversion's `.mtl`; f3c done: a Common `.model` converted in each linking player's Models task on PES 18-21, per-kit sets of either format, a pre-Fox Common texture path pointed at the Fox Common directory; g1 done: the pose measurement on the games' own files, the pre-check's rule settled as a blended-delta test; g2 done: `needs_conversion` is that test, `skf_*` pass through, tolerance 3e-3; g3a done: the Fox Models task pre-checks every selected FMDL with its `.skl`, the gloves task reads a hand-split part's `.skl`, a slotless conversion skeleton is dropped silently; g3b done: every pre-Fox `.model` pre-checked with the member's `.mtl` packed beside a moved one, the Common `.mtl` a link names read; 4.17's slices are done); 4.20 in progress (lead rulings and a done 2026-10-09: the gate gone, a file no role reads `file_not_used`; b1 done: unused kit stems, `shared_folder_no_model`, Common-set textures, the Common glTF order; b2 done: per-kit sets left out on pre-Fox where no `face.xml` names them, a shared folder's `face.xml` ignored; 4.20 done); 4.19d done 2026-10-09 (with 4.27's pre-Fox half; the TC-REF-04 in-game check: checked 2026-10-09 by the lead on PES 17 through the harness (`.tmp/4_19/ingame/`, results in `.tmp/4_0/apptest/results.txt`): the referee present with the pair installed, nothing drawn; a `.mtl` beside a nocloth `.model` is read and stops the model drawing (runs E and F against 2026-10-08's run I), so the pre-Fox marker goes by Red's route instead: step 4.19f, DECISIONS 2026-10-09; the scene where the pre-Fox marker shows is a maintainer question); 4.27b done 2026-10-09 (the Fox referee configs also their `UniformParameter.bin` entries; a refs-only Fox run writes the team side); 4.19f done 2026-10-09 (the pre-Fox marker by Red's route: the template prop's texture replaced, no collar pair, `collar_empty.model` gone); 4.19e done 2026-10-09 (a referee's shared link his slot's folder alone on PES 15-17; no face folder for a referee without a face model); every Phase 4 step is done (the 4.14 and 4.17 parent bullets closed 2026-10-09, their verify criteria re-run); 4.y-conv in progress since 2026-10-09: the lead's audit done (verify re-run, acceptance, catalog producers, sweep, `pub` census, Clef, the export census), 4.y-fix1 done 2026-10-09 (eight small fixes, three issues closed by tests); the whole-crate mutation runs of `aesthetics_export` and `pipeline` done and triaged, `team_compiler`'s next; 4.y-fix2 done 2026-10-09 (`export_layout_old`, the dual-engine face diff, the survivors' tests), 4.y-fix3 done 2026-10-09 (a cube-map DDS goes out as Red ships it; TC-TEX-13, acceptance 280 of 280), 4.y-fix4 done 2026-10-09 (S7's rework: a linked face's `.model` boots convert with their `.mtl`, a boots or gloves link combines with the player's effective parts; TC-MOD-58 to 60), 4.y-fix5 done 2026-10-09 (S8's rework: a Common model's local `.mtl` resolves its textures in the folder, every DDS kind the decoder refuses is `texture_codec_unsupported`, an FTEX portrait gets the full chain; TC-MOD-61, TC-TEX-14, TC-TEX-15), 4.y-fix6 done 2026-10-09 (a DX10-header DDS portrait goes out re-headered: the VGL stream's PES 19 crash; TC-PRT-04, acceptance 287 of 287), then the `duck` reviews (started 2026-10-09; Astra's five-hour quota stopped the first two mid-review, retried from 14:50), 4.c-threshold done 2026-10-09 (0.7 kept), 4.z-rewrite
reference exists (4.31 done: `tests/parity_prefox.rs`); 4.33, 4.34, 4.c-pass and
4.c-fix1 are done; 4.30,
4.5 to 4.8 and 4.10 to 4.13 are done (4.6c moved to Phase 8's cancellation). 2.5b (GPU BC7) is step 16.x (decision entries
2026-09-21 and 2026-09-28). Release target (2026-09-28): 0.1.0 after Phase 8; phase order 1–6,
8, 0.1.0, 7, 9–16 (`core/development_plan.md` "Releases"); first-class target the Fox version
the cup moves to around April 2027 ("Target versions").
**Blocked on:** nothing.

**Handover (2026-10-01).** Until the maintainer says otherwise, the session runs as a single
Claude agent with no sidekick and no reviewer of another model family. While that holds:

- **Since 2026-10-01 (maintainer):** the lead runs in Claude Code and delegates implementation
  to a same-family subagent (Opus from 3.8b, Fable from 3.9d on; 3.8a used Sonnet) under
  the lead/sidekick rules, to keep the lead's context small. That subagent is the sidekick, not
  a reviewer: the queue below still holds every cross-family checkpoint. From 2026-10-03, after
  step 4.6: the lead runs on Fable 5.1 and the sidekick on Opus 5.5 (the two have separate
  usage pools); which model suits which role is under evaluation, so the lead keeps a per-slice
  tally of the sidekick's deliveries in its project memory.
- **Implement directly**, but keep everything else `AGENTS.md` asks of a sidekick's work: read the
  plan section from the file first, paste the plan's code blocks, red-first tests (record each
  test's failing assertion in the commit's log line, since no report carries it), the honesty and
  design sweeps on your own diff, the gates, `mutants-diff` with every survivor triaged, one
  commit per reviewable slice (about 500 lines).
- **Cross-family reviews run through the `duck` skill** (Astra, with SWE-2 rounds around it)
  since 2026-10-09, when the maintainer gave the lead the means to run them itself; before
  that they were queued here, never skipped and never substituted (a same-family subagent
  shares the blind spots the review exists to catch). Astra has two quotas, a five-hour one
  that five or six reviews can empty and a generous weekly one (about forty reviews): when a
  call stops for quota, the review goes on the queue below (commit range, plan sections), the
  lead retries once an hour, and once a call works the queue runs until the next stop. Work
  continues past a queued review, but no phase closes with one outstanding (one exception:
  Phase 3 closed with its reviews queued, by the maintainer's directive of 2026-10-02). The
  backlog below (S7-S16 of `.tmp/lead/review_queue.md`, plus 4.14's (a) and (b) and 4.17's
  (c) and (d)) runs as the first half of Phase 4's converge, on the finished code grouped by
  module, by the maintainer's decision of 2026-10-09: the remaining slices (4.17g, 4.19d,
  4.20, 4.27) rewrite parts of what those reviews would read, and converge reviews the
  crate anyway.
- **Queue:** 3.7 (b): `crates/libs/pipeline` from its first commit, against `libs/pipeline.md`
  and `core/parallelism.md` "Memory budget"; the prior 3.6 rulings are in the log, and
  `.tmp/3_6/review_brief_3_6.md` is a template for the brief. 3.8 (b): `studio_core`'s `CliError`/
  `AppPaths`/location and the `team_compiler` CLI surface, from 3.8a's commit, against
  `core/architecture.md` "Tool plugin interface", `core/distribution.md` "Data location" and
  `team_compiler/settings.md` "CLI" (one review once 3.8 is done). 3.z (b): the shell slice,
  from the 3.z commit (new `pub` `StudioApp`, `run_gui`, `ToolContext::with_events`,
  `EventLines`; three crates), against `core/gui.md` "Shell layout" and "Event wiring",
  `core/architecture.md` "Tool plugin interface" and "Event system", `core/development_plan.md`
  "Phase 3" (the shell slice bullet) and the decision entry "3.z: the shell slice's edges".
  3.9f (a): TC-CLI-08 and TC-CLI-09 (`team_compiler/README.md` "CLI and output"), against
  `team_compiler/settings.md` "Path resolution", `team_compiler/messages.md` (`no_exports_found`
  and the settings/environment paragraph) and the decision entry "a missing exports folder is
  created or refused in plain words"; the 3.9f code rides along with it. 3.y (c): the converge
  reviewer loop, one surface per crate (`aesthetics_export`, `pipeline`, `team_compiler`, the
  Phase 3 parts of `studio_core` and `studio`), against the Phase 3 plan sections and the TC
  IDs, the lead's converge record in the 3.y row and its log line; after the loop ends, the
  second whole-crate `just mutants` per crate (`AGENTS.md` "Closing a phase").
  4.0 (a)-style: the plan rewrite of the decision entries "aesthetics travel in the database
  tables" and "Fox first; game-behavior changes go through FoxDen" (2026-10-03, its commit), against
  the step 4.0 row and `.tmp/4_0/apptest/results.txt` (the in-game evidence); the surfaces are the
  plan files those entries list, `settings_toml.md` and `save_editor.md` "Stripped save" first.
  4.1 (a): the Phase 4 Acceptance section (`team_compiler/README.md`, the Phase 4 scope paragraph
  and TC-PRT/MOD/TEX/CHK/XML/KIT-10../ROOT-06../CMN/BIN/PLN/REF/OUT-07../DEP) and the decision
  entry "Phase 4 itemization rulings", against `development_plan.md` "Phase 4" and the
  `pipeline.md` walkthrough, from the 4.1 commit.
  4.3-4.4 (b): `team_compiler` from `a949664` to `ed5e6a3` plus `aesthetics_export`'s
  `SharedModelFolder.path` (two crates, new `pub(crate)` shapes: `ModelFolder`, `TaskGroup`,
  `TextureHome`, `TaskFailure`), against `pipeline.md` steps 3, 6 and 7, `player_folders.md` "ID
  allocation", "Shared models" to "Merge constraint", TC-MOD-01..09, TC-PLN-01/02, and the
  decision entries of 2026-10-03 from "a player folder's tasks commit as one group" on.
  4.5c-4.6b (b): `team_compiler` from `624825f` to the 4.6b commit plus `aesthetics_export`'s
  `common_link_name` and `dds_convert`'s `encode_dds` (three crates, new `pub` items), against
  `player_folders.md` "Common model links and model merging", "Portraits", `model_format.md`
  "Link files", `pipeline.md` steps 5-6 and its "Common textures are one task" bullet,
  `messages.md` "Textures", `libs/dds_convert.md` "`dds_convert` API", TC-MOD-10/11,
  TC-TEX-01/02/04/06, TC-PRT-01, TC-KIT-10, and the decision entries of 2026-10-03 from "Fox's
  Common textures are one task" on.
  4.7 (a) and (b), the step now done: TC-CHK-06 and TC-CHK-07
  (`team_compiler/README.md` "Deep checks"); `aesthetics_export`'s `ContentFinding` and
  `ValidationReport::with_content_findings`, `team_compiler/src/deep/`, `validation.rs` and
  `reader/` (`ContentSource`, `route`), `fmdl`'s and `pes_model`'s `check::CODES`,
  `dds_convert::probe`, from `f937e3d` to the 4.7f commit (`perf(team_compiler)`, 2026-10-04),
  against `libs/pipeline.md` "What a solid `.7z` is charged", `pipeline.md` "2. Per-export serial steps" (deep format pass),
  `object_model.md` "Validation semantics" (content findings), `messages.md` "Model checks"
  and "Textures", and the decision entries "the deep pass: the compiler checks, the export
  crate drops" and "texture checks: from the header, in the deep pass".
  4.8 (b): `aesthetics_export`'s `colors_txt` (`read_colors_txt`, `ColorsTxt`,
  `RefusedColorLine`, `ColorLineRefusal`; new `pub` items) and `validate_with`'s two
  filtered fields, `team_compiler`'s `bins/`, `deep/documents.rs` (`colors_findings`),
  `validation::team_colors`, the kit task's colors and the writer's two bins, from
  `7d3d94a` to the 4.8c commit (`feat(team_compiler)`, 2026-10-04), against `pipeline.md`
  "4. Per-export non-model steps" (Bins accumulation) and "Resolved decisions" (Kit colors
  fallback), `player_folders.md` "Root files" (Colors), `libs/color_tools.md` "Dominant
  kit-color extraction", `resources/bins/README.md`, TC-KIT-11..14, TC-ROOT-10,
  TC-BIN-01..03, and the decision entries "kit colors merge into a team's UniColor record
  by kit number" and "kit colors derive from the effective main texture; the missing
  pair's bytes".
  4.9a (b): `fpc` (`kit_values()`), `kit_config` (`apply_fpc`, `matches_fpc`: changed
  `pub` signatures) and `team_compiler` (`plan::EffectiveTeamKitFpc`, the kit task's
  reconciliation, `deep/documents.rs` `kit_config_findings`), the 4.9a commit
  (`feat(team_compiler)`, 2026-10-04), against `fpc_toggle.md` "Team kit-FPC status and kit
  configs", `libs/fpc.md`, `pipeline.md` "4. Per-export non-model steps" (Kits),
  `messages.md` (`kit_config_version_clamped`, `kit_config_fpc_adjusted`), TC-KIT-15..17
  and the decision entry "4.9a: the FPC kit values take no version; the template carries
  them".
  4.10 (b): `dds_convert` (`resize`, a new `pub`) and `team_compiler`
  (`processing/kit_layout.rs`, the kit task's re-layout), the two 4.10 commits
  (`docs(team_compiler)` and `feat(team_compiler)`, 2026-10-04), against `pipeline.md` "4.
  Per-export non-model steps" (Kits, "Layout conversion"), `player_folders.md` "Kit layout
  marker", `libs/dds_convert.md` "`dds_convert` API", `messages.md`
  (`kit_layout_converted`), TC-KIT-18, TC-KIT-20, `tests/fixtures/kit_layout/README.md`,
  and the decision entries "the kit layout table covers the socks alone, in two bands" and
  "the suite's one resampler is `dds_convert::resize`". The measurement scripts
  (`scripts/provenance/kit_uv/`) are part of the surface: the table is only as good as they
  are. 4.11 (b), one review over the step's slices as they land: 4.11a, `dds_convert`
  (`encode_png`, a new `pub`) and `team_compiler` (`processing/team_assets.rs`,
  `TaskKind::Logo`, `paths::logo`), the `feat(team_compiler)` commit of 2026-10-04 for
  4.11a, against `pipeline.md` "4. Per-export non-model steps" (Logo), `player_folders.md`
  "Logo", `messages.md` (`logo_fit_applied`, `logo_upscaled`), TC-ROOT-06 to TC-ROOT-08 and
  the decision entry "the logo's geometry, its findings and its PNG encoder"; 4.11b,
  `team_compiler` (`output/teamnotes.rs`, the note's path from `validation.rs` through
  `plan_run` to `compile::run`), its `feat(team_compiler)` commit, against `pipeline.md`
  "2. Per-export serial steps" (5, Notes collection), `messages.md`
  (`teamnotes_write_failed`), TC-ROOT-09 and the decision entry "`teamnotes.txt`: its
  layout, when it is written, and its failure"; 4.11c, `team_compiler` (`kit_variants.rs`,
  `complete_kit_variants`, `point_texture`, `PlayerFile::LeftOutKitVariant`), its
  `feat(team_compiler)` commit, against `model_format.md` "Kit-dependent assets (`kitN`)",
  `pipeline.md` "4. Per-export non-model steps" (Kit-dependent assets), `messages.md`
  (`kit_variant_missing`, `kit_variant_model_fox`), TC-CMN-04, TC-CMN-05 and the decision
  entry "kit variant sets are found from the files, and completed by the textures task".
  4.11d, `team_compiler` (`PlayerFile::CommonTexture`, `point_texture`'s places), its
  `feat(team_compiler)` commit, against `model_format.md` "Link files (`.common`)",
  `pipeline.md` "3. Per-model-folder parallel steps" step 6, TC-TEX-09 and the two "Issues"
  entries the slice opened.
  The acceptance section changed with these slices (TC-TEX-09 and TC-CMN-07 added,
  TC-CMN-05 narrowed to its Fox half): they join the queued (a) review of the section.
  TC-MOD-43 (the pre-Fox hand split, written 2026-10-08 for 4.14e3) joins it too.
  4.14e3 (b): `model_convert` (a `pub` `.model` hand-weight detector beside
  `fox_has_hand_weights`) and `team_compiler` (the deep pass recording it, the face task's
  split), its `feat(team_compiler)` commit, against `model_conversion/hand_split.md`
  "Pipeline integration", `messages.md` `model_hand_split`, TC-MOD-43 and the decision
  entries of 2026-10-08 on the pre-Fox split.
  4.12 (b): `team_compiler` (`plan/subset.rs` `model_role` under the marker,
  `PlayerFile::UnusedFaceFile`, `FolderModels::of_player_files`; `plan/mod.rs`
  `ModelFolder::ingame_face`, `folder_tasks`' blank face; `validation.rs`
  `face_file_not_used`), its `feat(team_compiler)` commit of 2026-10-04, against
  `player_folders.md` "`ingame_face` marker" and "`ingame_face` with shared links",
  `pipeline.md` "2. Per-export serial steps" step 4, `messages.md` (`face_file_not_used`),
  TC-MOD-16 to TC-MOD-19 and TC-MOD-32, and the two decision entries of 2026-10-04 on the
  face file with no face model and on TC-MOD-09 (reworded; TC-MOD-32 is new: both join the
  queued (a) review of the section).
  Coverage tag and `name.y` (a): the scenarios written on 2026-10-04 (TC-ID-05 to 07,
  TC-KIT-24, TC-BIN-14 to 17 new; TC-ID-01, TC-KIT-17, TC-BIN-05, 06 and 15, TC-PLN-03
  reworded) with `aesthetics_export/object_model.md` "Coverage tag", `pipeline.md` "Bins
  accumulation" and the two decision entries of that date join the queued (a) review.
  4.13 (b), one review over the step's slices as they land: 4.13a, `team_compiler`
  (`validation.rs` `refuse_duplicate_teams`), its `feat(team_compiler)` commit of
  2026-10-04, against `pipeline.md` "3. Per-model-folder parallel steps" (the run-level
  paragraph), `messages.md` (`duplicate_aesthetics_export`), TC-PLN-03, 05 and 07 and the
  decision entry "`duplicate_aesthetics_export` is the validation pass's"; 4.13b,
  `team_compiler` (`plan/overrides.rs`, `output/writer.rs`, `compile.rs`), its
  `feat(team_compiler)` commit of 2026-10-04, against `pipeline.md` "5. Writer" step 1,
  `settings.md` "Path resolution" (`overrides/`), `messages.md` (`duplicate_path`,
  `overrides_active`), TC-PLN-04 and the decision entry "the `overrides/` tree is the
  writer's; no path preflight over the tasks"; 4.13c, `team_compiler` (`reader/source.rs`
  `SourceRevision`, `compile.rs` `plan`, `build`, `coordinate`), its `feat(team_compiler)`
  commit of 2026-10-04, against `pipeline.md` "Resolved decisions" (Source snapshot),
  `messages.md` (`source_changed_during_run`), TC-PLN-06 and the decision entry "a
  source's revision is the listing's, checked after each task's read".
  4.33 (b), `kit_config` (`binary.rs` Name Y, `model.rs` `field_limits`, `UNKNOWN_MASKS`)
  and `team_compiler`'s tests, its commit of 2026-10-07, against `kit_config_editor.md`
  (layout rows 0x1C and 0x1D, "Version differences"), TC-KIT-17 and 24 and the decision
  entry "`name.y` is PES 21's value on every version; the template stays".
  4.34a (b), `aesthetics_export` (`parse/identity.rs` `ExportCoverage`, `coverage`; the
  draft's and validated export's `coverage`; `export_tag_missing` in `validate/mod.rs`) and
  `team_compiler` (catalog, help), its commit of 2026-10-07, against
  `aesthetics_export/object_model.md` "Coverage tag" and the plan's `ExportCoverage` and
  `coverage` blocks, `messages.md` (`export_tag_missing`), TC-ID-05 to 07; the test renames
  are mechanical (`Midcup` by default) and need no review.
  4.34b (b), `aesthetics_export` (`validate/kits.rs` `missing_kinds`, `kit_folder`; the
  removed second kit drop in `validate/mod.rs`) and `team_compiler` (`bins/mod.rs`
  `keep_kits`, `edit_kits`; `plan/mod.rs` `full_team_kits`; `output/writer.rs` `finish`),
  its commit of 2026-10-07, against `pipeline.md` "Bins accumulation", `object_model.md`
  `KitsFolder`, TC-BIN-14, 15, 19, 20 and the decision entry "a `Full` export gets an empty
  `p1/` and `g1/` where it has none".
  4.21a (b), `team_compiler` (`bins/dpfl.rs`, `bins/installed.rs`, `WorkingBins`,
  `compile.rs` `working_bins`, `messages.rs` `ErrorOrWarning` and `deploy_message`) and
  `studio_core` (`CommonSettings::pes_folder`, new `pub`), its commit of 2026-10-07, against
  `pipeline.md` "Bins accumulation" and the "DpFileList upgrade" sub-bullet on the format,
  `messages.md` (`dpfilelist_missing`, `installed_bin_unreadable`, `bin_source`),
  `settings.md` `pes_folder_path`, TC-BIN-05, 08, 09, 13, 21 and the decision entry "the
  working-bin walk's edges, and the measured DpFileList layout".
  4.21b (b), `team_compiler` (`templates.rs` `Templates`, `Resource`, `RESOURCES`; its reach
  into `compile.rs`, `bins/`, `processing/`), its commit of 2026-10-07, against `pipeline.md`
  "Resolved decisions" ("Templates and fallback bins"), `messages.md`
  (`template_override_unreadable`, `template_override_active`), TC-BIN-07 and the decision
  entry "`templates/` overrides: flat names, read first, unreadable aborts".
  4.21c (b), `team_compiler` (`bins/kit_configs.rs`, `UniColorBin::kits`, `kit_slot`,
  `WorkingBins` parsed, `plan/mod.rs` `TeamKits`, `output/writer.rs` `finish`,
  `templates.rs` `Format`), its commit of 2026-10-07, against `pipeline.md` "Bins
  accumulation", `fpc_toggle.md` "Kit slots absent from the export", `messages.md`
  (`kit_config_fpc_adjusted`, `kit_config_fpc_unpatched`, `installed_bin_unreadable`,
  `template_override_unreadable`), TC-BIN-06, 16 and the decision entry "FPC patching takes
  the slots from `UniColor.bin`; bins parse as they are read".
  4.22 (b), `team_compiler` (`bins/player_tables.rs`, `plan/item_rows.rs`, the walk's three
  tables, `output/writer.rs` `add_player_tables`, the parity test's warnings), its commit of
  2026-10-07, against `pipeline.md` "Bins accumulation" (player appearance tables, the walk's
  edges), "Game paths reference", `messages.md` `player_table_missing`, TC-BIN-10, 11, 12, 17,
  22 and the decision entry "a player table no installed CPK holds is not written".
  4.29a (b), `fmdl` (`ops/paths.rs` `used_texture_paths`) and `team_compiler`
  (`bins/installed.rs` `InstalledPaths`, `processing/model.rs` `texture_supply`, `messages.rs`
  `ErrorUnlessKept`, `plan/mod.rs` shared folders' Common stems), its commit of 2026-10-07,
  against `pipeline.md` "Resolved decisions" ("A texture a model names must exist"), "Bins
  accumulation" (the walk opens every listed CPK), `messages.md` `fmdl_texture_not_found`,
  TC-TEX-05, TC-CMN-06 and the decision entry "what 'a texture a model names must exist'
  compares, and where it runs".
  4.29b (b), `aesthetics_export` (`ValidationContext.installed_common_textures`,
  `validate/links.rs` `common_target_missing`) and `team_compiler` (`validation.rs`
  `installed_common_textures`, `bins/installed.rs` `walk`'s `take` closure and
  `installed_paths`, `check.rs`), its commit of 2026-10-07, against `pipeline.md` "Resolved
  decisions" ("A texture a model names must exist", the texture link), `object_model.md`
  "Validation semantics" and its `ValidationContext` block, TC-TEX-11 and the decision entry
  "`ValidationContext` carries the installed Common texture stems".
  4.23a (b), `team_compiler` (`output/sink.rs` new, `output/writer.rs` through the sink,
  `compile.rs` `OutputMode` and `promote`, `cli.rs` `output_mode`, `output/deploy.rs`
  `promote_livecpk`), its commit of 2026-10-07, against `pipeline.md` "5. Writer" step 5
  (sideload, staging), `settings.md` "CLI" (the sideload refusals), TC-OUT-09, TC-OUT-10 and
  the decision entry "the sideload tree is staged, and 4.23 lands in two slices".
  4.23b (b), `team_compiler` (`processing/materialize.rs` new, `processing/model.rs` `package`
  returning the package's files, `processing/mod.rs` `CompileContext.target`, `compile.rs`
  `OutputMode::Test`, `output/writer.rs` `bins_prefix`, `output/deploy.rs` `promote_tree`),
  its commit of 2026-10-07, against `pipeline.md` "5. Writer" step 5 (the materialize
  paragraph), TC-OUT-07, 11, 17 and the decision entries "where a test-mode entry goes" and
  "the parity test reads the CPK, not a loose tree".
  4.24 (b), `team_compiler` (`output/deploy.rs` `preflight`, `deploy`, `Staging` and `sweep`,
  `compile.rs` `promote` and `deploy_failed`, `messages.rs` five codes, the tests' harness
  pointing at a sandbox PES folder), its commit of 2026-10-07, against `pipeline.md` "6.
  Post-processing" (Staging, Deploy CPKs, Degraded run, Destination writability preflight),
  `messages.md` "Output stage and savefile", TC-DEP-01 to 07 and the decision entry
  "deployment: no marker file, failures named by step, a lock per staging folder".
  4.25b (b), `team_compiler` (`templates.rs` `DPFILELIST` and `official_list`,
  `output/deploy.rs` `preflight`'s comparison, `not_official`, `cpks_missing`, `messages.rs`
  three codes, the tests' installs upgraded), its commit of 2026-10-07, against `pipeline.md`
  "6. Post-processing" (DpFileList upgrade, Destination writability preflight), `messages.md`
  the three `dpfilelist_*` rows, TC-DEP-05, 12, 14 and the decision entry "the official-list
  check: only a compile that deploys, and its findings' context".
  4.25c (b), `team_compiler` (`upgrade.rs` new: `plan`, `run`, `size_text`; `cli.rs`
  `download_folder`; `templates.rs` `PLACEHOLDER_CPK`, `read_reported` moved from
  `compile.rs`; `messages.rs` six codes), its commit of 2026-10-07, against `pipeline.md` "6.
  Post-processing" (DpFileList upgrade, the rename and placeholder sub-bullets), `settings.md`
  "CLI" (`upgrade-dpfl`), `messages.md` the six `upgrade-dpfl` rows, TC-DEP-09, 10, 13 and the
  decision entries "a stem's renames include its files already under official names" and
  "what `upgrade-dpfl` reports, and the cases the plan left open".
  4.26a (b), `cpk` (`write.rs` `CpkWriter::len_with`, the shared `tables` layout), its commit
  of 2026-10-07, against `pipeline.md` "Multi-CPK mode: teams parts" (the first-fit bullet:
  the cap checked TOC included) and the decision entry "multi-CPK mode: the official list's
  slots, permits given back, all-or-nothing install".
  4.26b (b), `team_compiler` (`output/parts.rs` new: `slots`, `TeamsParts`, `Unplaced`;
  `compile.rs` `CpkLayout`, `staged_output`, `size_over_limit`, `promote` over several
  CPKs; `output/writer.rs` the parts path and `admit`; `cli.rs` `compile_settings`;
  `settings.rs` three keys and `boundary_cpk_name`; `messages.rs` three codes and
  `size_text`), its commit of 2026-10-07, against `pipeline.md` "5. Writer" step 6 (Multi-CPK
  mode), `settings.md` (the four multi-CPK rows), `messages.md` the three size rows and
  `deploy_skipped_by_flag`, TC-OUT-12..16 and the decision entries "multi-CPK mode: the
  official list's slots ..." and "`cpk_part_max_size` is a byte count ...".
  4.26c (b), `team_compiler` (`output/deploy.rs` `preflight` over several CPKs,
  `deploy`, `install_all`, `undo`, `DeployFailure::Rename { cpk, error }`; `compile.rs`
  `CpkLayout::cpks`, `promote`; `output/parts.rs` the across-parts duplicate check;
  `cli.rs` the bins-stem refusal), its commit of 2026-10-07, against `pipeline.md` "6.
  Post-processing" (Deploy CPKs, the all-or-none paragraph), `messages.md`
  `dpfilelist_outdated`, `cpk_name_unlisted`, `old_cpk_locked`, TC-DEP-08, TC-DEP-11 and the
  decision entries "several CPKs: one preflight finding each ..." and "a teams stem that is
  the bins CPK's own is refused".
  4.19a (b), `team_compiler` (`paths.rs` `PackageKey`, `REFEREE_TEAM_ID`; `plan/subset.rs`
  `referee_not_compiled`, `link_feeds_own_package`; `plan/ids.rs`; `plan/mod.rs` `plan_run`'s
  refs branch, `mapped_folders`, `player_folders`; `plan/item_rows.rs` `row_id`), its commit
  of 2026-10-07, against `blue_port.md` "Referee export processing", `pipeline.md` step 6
  (texture relocation), "Game paths reference" referee rows, the README gate paragraph's
  "Step 4.19 lifts" sentences, TC-REF-03, 05, 10 and the decision entry "a refs export's
  kits, logo and portraits are named by the gate ...".
  4.19b (b), `team_compiler` (`output/writer.rs` `RefsCpk`, `Written`, `with_refs`,
  `overridden`; `compile.rs` `run`'s discovery, `CpkLayout::cpks`/`promoted` with `refs`,
  `referee_tasks`, the written filter; `cli.rs` `compile_settings`, `parts_layout`,
  `shared_with_team_side`; `bins/installed.rs` `walk`, `installed_paths`; `validation.rs`
  `validation_pass`; `reader/mod.rs` `ExportSource::is_referees`), its commit of 2026-10-07,
  against `pipeline.md` "5. Writer" step 5, `settings.md` `refs_cpk_name`, TC-REF-01, 02,
  TC-DEP-11 and the decision entry "the refs CPK beside the team side ...".
  4.19c (b), `team_compiler` (`templates.rs` `referee_tree!`, `REFEREES_FOX`, `tree_files`,
  `referees_fox`; `output/writer.rs` `Referees`, `finish_referees`; `compile.rs` `build`'s
  referee routing per mode), its commit of 2026-10-07, against `pipeline.md` "Resolved
  decisions" (templates embedded, the referee trees' override names), "Output-mode
  artifact routing", TC-OUT-09 and the decision entry "the referee template tree: embedded
  per file ...".
  4.27a (b), `team_compiler` (`processing/referee_marker.rs`, `processing/kit.rs`
  `reserved_collar_field`, `output/writer.rs` `Referees.marker` and `wearing_marker`,
  `paths.rs` `collar`, `templates.rs` `REFEREE_MARKER`, `plan/mod.rs`
  `TaskKind::RefereeMarker`) and `resources/templates/referee_marker.fmdl`, its commits of
  2026-10-07, against `blue_port.md` "Referee export processing", `messages.md`
  `kit_collar_reserved`, TC-REF-06, 07, 08 and the decision entry "the Fox referee marker
  ...".
  4.9b1 (b), `aesthetics_export` (`validate/folders.rs` `check_collars`) and `team_compiler`
  (`deep/collar.rs`), its commit of 2026-10-07, against `pipeline.md` "Collars" and
  "Collar contract", `messages.md` `collar_id_invalid`, `collar_id_conflict`,
  `object_model.md`'s `Collars/` allowlist row, TC-CMN-02, TC-CMN-10, TC-REF-07 and the
  decision entry "collars: the 9xx IDs, ...".
  4.9b2 (b), `team_compiler` (`plan/collars.rs`, `plan/subset.rs` `collar_file`,
  `TeamKitEdits`, `TaskKind::Collar`, `bins/kit_configs.rs` `edit_absent_slot`,
  `edit_config`, `processing/kit.rs`), its commit of 2026-10-07, against `pipeline.md`
  "Collars", "Collar contract", "Bins accumulation", `fpc_toggle.md` "Kit slots absent
  from the export", TC-CMN-01, TC-CMN-03 and the decision entries "collars: the 9xx IDs,
  ..." and "a refs export's collar file is named by the gate".
  4.28 (b), `pipeline` (`MemoryBudget::charge`, `peak`) and `team_compiler`
  (`CompileContext.budget`, `texture::decode_charge` and its call sites,
  `TaskBatch.output`, `writer.rs` `commit`), its commit of 2026-10-07, against
  `libs/pipeline.md` "Memory budget", `pipeline.md` "Admission" (Charges while a task
  runs), `core/parallelism.md` "Memory budget" and the decision entry "memory
  accounting: a running task charges without waiting".
  4.18 (b), `model_convert` (`ops::hand_split::fox_has_hand_weights`) and `team_compiler`
  (`deep/` `ContentPass`, `validation.rs`, `plan/mod.rs` `hand_split_parts`,
  `folder_tasks`, `TaskKind::files`, `processing/model.rs` `parts_of`, `split_fmdl`), its
  commit of 2026-10-07, against `model_conversion/hand_split.md`, `pipeline.md` "2.
  Per-export serial steps" step 6 and "3. Per-model-folder parallel steps" step 3,
  `messages.md` `model_hand_split`, TC-MOD-31 and the decision entry "hand auto-split:
  face models only, ...".
- For the lead, on return: the review process on trial (3.1) opens with a full sidekick review
  loop, then runs GPT's loop with a full sidekick loop after each GPT round, calling GPT again
  only once that sidekick loop has ended and GPT's own loop has not; not yet in `AGENTS.md`
  (3.6: GPT 4 of 7 accepted, then sidekick S1 3 of 7, so both loops ended after one round
  each). Prior rulings: `.tmp/3_6/review_rulings_3_6.md`. From 2026-10-04 the Devin lead runs the
  whole queue below on the code as it now is (a diff a later step superseded is not reviewed
  on its own), in queue order; its progress is `.tmp/lead/review_queue.md`.
- Paused again on 2026-10-06 (maintainer). Status:
  - **Reviews done:** S1-S6 (`pipeline`, `studio_core` + `studio` Phase 3,
    `aesthetics_export`, the `team_compiler` Phase 3 skeleton, the 4.0 plan rewrite, the
    Phase 4 Acceptance section).
  - **Still queued:** S7-S16, from 4.3-4.4 to the Phase 3 close mutants. Each is mapped to
    its queue item in `.tmp/lead/review_queue.md`.
  - **Code that lands meanwhile** joins the queue as usual. The S7+ reviews run on the code as
    it is then.
  - **Before 4.22 and 4.27:** read the 2026-10-06 test results in "Issues". The boots drop is
    safe on Fox, and on Fox the referee marker stays `collar_077`. Fox referee configs go in
    as loose files and `UniformParameter.bin` entries both (4.27b). Still open: the PES 17
    collar.
  - **4.9's stock-set rule stays** until Test 3 is run (postponed).
  - **In-game tests:** the scratch tool and scripts are in `.tmp/4_0/apptest/` (see
    `.tmp/lead/review_queue.md` "RESUME HERE"). Each test has its own GUIDE and install/revert
    script; nothing is shared with the workspace build.
- Decisions the maintainer must make (new dependencies, `unsafe` outside the listed sites, game-
  or format-facing behavior the plan does not settle) are still asked, not decided.

---

## How to resume

| Thing | Where |
|---|---|
| Plan index | `docs/plans/README.md` → `docs/plans/core/README.md` |
| Development phases | `docs/plans/core/development_plan.md` "Development Plan" |
| Legacy tools (format evidence) | `docs/plans/core/README.md` "Project context" (paths are per-machine) |
| Skeleton data | `resources/skeletons/` (see its README) |
| FPC source text (for the `fpc` crate) | `resources/FPC.wikitext` |
| `.model` format as measured (for the plugin author; the crate's notes are in `libs/README.md`) | `resources/prefox_model_format.md` |
| Coding rules and verification gates | `docs/CONTRIBUTING.md` |
| Domain terms | `docs/GLOSSARY.md` |

### Current-state gotchas

Things that are true *right now* and would cost the next session time to rediscover (a broken
fixture, a flaky test, a crate that doesn't build on one platform). Remove an entry when it stops
being true. Permanent toolchain facts go in `docs/CONTRIBUTING.md` "Toolchain notes" instead.

- `just` needs PowerShell as its Windows shell (`set windows-shell` in the justfile): Git for
  Windows' `sh` is not on PATH from a plain PowerShell. Recipes still stop at the first failure.
- `cargo install just cargo-deny` is required once per machine (both take a few minutes to build);
  the pinned toolchain and the wasm32 target install themselves on the first `cargo` call.
- The lockfile holds egui 0.36.1 on purpose (0.36.2 was under a week old at bootstrap); a bare
  `cargo update` would move it. Bump deliberately.
- `dds_convert`'s CPU BC1 core (`block_compression`, PCA plus one refinement) trails DirectXTex
  by about 2.5 mean error on an 8x4 grey-ramp mip while matching it within 1.0 at the top mip;
  the encode tests compare the chain-wide mean for that reason. Revisit with representative kit
  and face textures when the Team compiler can produce them (plan: "tune against representative
  textures").
- `just mutants-diff` reuses `mutants.out/`: it wipes a whole-crate run's results. Keep the
  whole-crate run's summary aside (to `.tmp/mutants_<step>_whole/`, only `*.txt` and
  `outcomes.json`: `AGENTS.md` "An archived run keeps its summary") before the first diff run.
- `block_compression` 0.10's BC3/BC4/BC5 decoder truncates the alpha-ramp interpolation where
  DirectX rounds; `dds_convert::dds::fix_interpolated_channels` re-derives those channels. Drop
  it if a later release rounds (the exact-match test will say).

---

## Phases

Status: `todo` · `in progress` · `done` · `blocked`. A phase is done only after its last two
steps, which every phase has: **converge** (the lead's own audit first, then the cross-family
reviewer's, both against the phase's plan sections and acceptance IDs; each gap becomes a new
step above it, and the phase waits for them) and **rewrite**
(the phase's plan sections rewritten in the present tense, in place). Then collapse its step list
below to this one row, keeping its `manual: checked` lines; the step-level detail stays in git history. Tool phases (3–6, 8–15, 17, 19, 20) also
open with an **acceptance** step: the tool plan's "Acceptance" section for that phase, written
before any code (GUI scenarios that no automated test can prove are marked `manual` and proven by
a recorded check at converge — `CONTRIBUTING.md` "Testing"). Procedure: `AGENTS.md` "Working
documents".

| Phase | Scope | Crates | Status |
|---|---|---|---|
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree`, `pes_version` | done |
| 2 | Library crates (standalone-verifiable) | `wezlib` `cpk` `fpk` `ftex` `dds_convert` `fmdl` `pes_model` `uniparam` `fox2` `archives` `fpc` `teams_list` `kit_config` `color_tools` `elevation` `model_convert` (native) `pes_savefile` `python_bindings` | done |
| 3 | Team compiler skeleton | `team_compiler`, `aesthetics_export`, `pipeline` | done (reviews queued) |
| 4 | Processing logic | `team_compiler` (`plan/` `processing/` `bins/` `output/`), `aesthetics_export` deep validation | todo |
| 5 | Savefile integration | `save_editor` logic, `aatf`, `team_compiler` `output/savefile.rs` | todo |
| 6 | Export upgrader | `export_upgrader` | todo |
| 7 | glTF support (runs after Release 0.1.0) | `model_convert` (glTF half) | todo |
| 8 | GUI | `studio_core` shell, `studio`, tool views, `color_tools` widget | todo |
| 0.1.0 | Release 0.1.0, after Phase 8 (`core/development_plan.md` "Release 0.1.0") | `just release`, `CHANGELOG.md`, help chapters, update check + notice, teams-list merge, maintenance mode | todo |
| 9 | Stadium compiler | `stadium_compiler` | todo |
| 10 | Music tools | `music_player`, `music_export_editor`, `music_export`, `audio_engine` | todo |
| 11 | Match tracker | `match_tracker`, `match_feed` | todo |
| 12 | Kit config editor | `kit_config_editor` | todo |
| 13 | Refs arranger | `refs_arranger` | todo |
| 14 | Balls compiler | `balls_compiler` | todo |
| 15 | Player aesthetics editor | `player_aesthetics_editor` | todo |
| 16 | Polish and distribution | — | todo |
| 17 | Team creator (post-release) | `team_creator` | todo |
| 18 | Studio Web (post-release) | — | todo |
| 19 | DB generator (scheduled by need, after 2 and 8) | `db_generator`, `pesdb`, `pes_savefile` `ops/populate.rs` | todo |
| 20 | Ruleset editor (post-release) | `ruleset_editor`, `aatf` writer | todo |

---

## Steps

`[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked / needs a decision

A step is done when its own `→ verify:` check has actually been run and passed, on top of the
gates. Every step carries one when itemized; if a step is listed without one, the agent writes it
before starting the step (a specific check, not "test it") and puts it in the step text. Mark a
done step with a one-line summary and the files or crates touched. One `[~]` per agent at a time.

### Phase 1 — Workspace bootstrap + core skeleton

Done 2026-09-13 (spec now describes what exists: `docs/plans/core/development_plan.md` "Phase 1"). Step detail
in git history up to commit `2f158a9`. CI proof: green run on `5e6ffc6`, deliberately red run on
`eccf0f3` (both `gates` jobs failed at `just gates`, `deps-check` unaffected), reverted in
`2f158a9`.

### Phase 2 — Library crates

Done 2026-09-30 (spec now describes what exists: `docs/plans/core/development_plan.md`
"Phase 2", `libs/README.md`, `model_conversion/README.md`, `pes_savefile/README.md`). Step
detail (2.1-2.21: each crate's build, review rounds A-C, the 2.20a-k converge with its
mutation runs, censuses and reviewer rulings) in git history up to commit `7de61f7`. Carried
forward: 2.5b GPU BC7 is step 16.x, the `dds_convert` cache bound is step 4.y, and the
phase's open questions are under "Issues".

### Phase 3 — Team compiler skeleton

Done 2026-10-02, its cross-family reviews still queued (the maintainer's directive; "Handover"
lists them, 3.7's and 3.8's among them; what they find becomes steps under this heading). Spec
now describes what exists: `docs/plans/core/development_plan.md` "Phase 3",
`team_compiler/README.md` ("Acceptance": 77 scenarios, 75 cited by tests, 2 manual),
`team_compiler/pipeline.md` "Run driver shapes (Phase 3)", `aesthetics_export/object_model.md`,
`libs/pipeline.md`. Step detail (3.1-3.9f, the 3.w `.cargo/mutants.toml` audit, the 3.y
converge with its whole-crate mutation runs, the 3.z shell slice, 3.x this rewrite) in git
history up to commit `32261dd`. Carried forward: the phase's open issues under "Issues".

TC-GUI-01 manual: checked 2026-10-02 by the maintainer on Windows, the window, the sidebar's
Team compiler entry, its Settings, Compile button and log.
TC-GUI-02 manual: checked 2026-10-02 by the maintainer on Windows, the /egg/ tracer export
compiled from the window, the log showing `egg Tracer: Info export_identified (team=/egg/,
id=792)` then `Run finished: exit code 0`. Linux: not yet checked, no machine with a display.

### Phase 4 — Processing logic

Itemized 2026-10-03 (step 4.1). Each step is one reviewable slice (about 500 lines of new
non-generated code; fixtures, tables and tests do not count); "Plan" names the section(s) the step
implements; `ae` is `aesthetics_export`, `tc` is `team_compiler`. A `(lead first: ...)` note is
plan text the lead writes before briefing the step; `(waits on the maintainer: ...)` names an input
only the maintainer has (listed again under "Phase 4 open questions"). Test data: `/co/` is team
714 (`teams_list.txt`), so its 40-ID block is 621-660, slot 05 is player 71405 with exclusive
boots/gloves ID 625 and the first shared ID is 644; `/egg/` is 792 (the tracer fixture, slot 05 =
79205 / 3745); `/a/` is 702, `/b/` is 707:

- [x] 4.0a PES12 re-study (maintainer, 2026-10-01): the stashed stub (`git stash` entry `pes12`,
  read with `git show 'stash@{0}^3:docs/plans/pes12.md'`, never popped or applied) checked against
  the code and `Tools_4cc/pes12tools` (`058398f`) and `PES-Tools`. Done 2026-10-02: the maintainer
  kept the road open for a third engine (`Engine` matched exhaustively, step 4.0c; the output sink
  over output-relative paths) and nothing PES 12-specific enters the plans (decision entry "the
  road stays open for a third engine"). Corrections for when the stub is restored: pes12tools
  reads Red-format exports (`pes12_import_team.py` walks `Faces/`, `Kit Textures/`), not Studio's;
  its main kit path is a draw-time UV remap (`runtime/kitforce.h`), the 1024x512 repaint the
  fallback; licensing is moot (its author allows free use, and the suite only reproduces results);
  its runtime and ID scheme may change as PES 12 modding progresses (CPK-like files, custom team
  IDs), so the stub's runtime, ID and bins sections are to be re-checked then
- [x] 4.0c `Engine` matched exhaustively (decision entry "the road stays open for a third
  engine"; `CONTRIBUTING.md` "Closed sets are enums"): `PesVersion::engine()` a `match`, and the
  `==`/`!=` comparisons on `Engine` in `aesthetics_export` (`validate/folders.rs`),
  `team_compiler` (`plan/mod.rs`, `plan/subset.rs`), `model_convert` (`convert.rs`) and
  `dds_convert` (`encode.rs`) rewritten as `match`es; no behavior change → verify: `just gates`,
  `mutants-diff` over the step, and a grep finding no `==`/`!=` on `Engine` left. Done: the six
  sites and `engine()` are `match`es, `engine_of_every_version` tests all seven; gates green, 77
  of 77; `mutants-diff f678d4f`: 13, 11 caught, 2 unviable; the grep finds none
- [x] 4.0b Sider feasibility study (maintainer, 2026-10-02): `c:/Data/4cc/0Tools/sider`. Done
  2026-10-02: the Team compiler's `--mode sider` becomes `--mode sideload` and Red's `sideload/`
  folder `overrides/`, with a plain **Launch PES** button beside Compile in sideload mode (decision
  entry "sideload mode and the overrides folder"). The from-scratch LiveCPK replacement the
  maintainer asked for is superseded by 4.0e: FoxDen already serves it
- [x] 4.0e Sideloading through FoxDen (maintainer, 2026-10-03; study notes outside git,
  `.tmp/4_7/sider_info/`): no Studio sideloader; sideload mode writes `{pes_folder_path}\livecpk\`,
  served by FoxDen on 18–21 and the user's Sider 3 on 17, refused on 15/16; `sideload_output_path`
  dropped; the untracked `docs/plans/sideloader.md` stub deleted; `fox_hook` → FoxDen in the
  Refs arranger plan; the PES 2020 history recorded (decision entry "sideloading through FoxDen")
  → verify: no `sideload_output`, `Sider's LiveCPK` or `fox_hook` (outside the "formerly" note)
  left in `docs/plans`, `GLOSSARY.md` or `crates/`; `cargo test -p team_compiler`. Done: the grep
  finds only the "formerly" note; `cli.rs` `Mode::Sideload` doc comment; 171 tests, fmt, clippy
  clean; acceptance 77 of 77. FoxDen gaps forwarded to its maintainer: under "Issues"
- [x] 4.0d The 4.0b renames in the code: `team_compiler` `Mode::Sider` → `Mode::Sideload`
  (`--mode sideload`), its help topic and tests → verify: `just gates`; the refusal test asserts
  `--mode sideload`; no `sider` left in `crates/tools/team_compiler`. Done: `cli.rs` `Mode::Sideload`,
  `tests/cli/preflight.rs`, `help/01_command_line.md`; gates green, 77 of 77; `mutants-diff
  b8e7393`: no mutants (a rename)
- [x] 4.0 **GATE, maintainer only: in-game appearance-fallback test.** No agent itemizes Phase 4,
  writes a Phase 4/5/6 Acceptance section, or starts Phase 4/5/6 work until the maintainer has
  run the test and reported the result. The idea under test: a savefile player whose appearance
  record's player ID is -1 (`0xFFFFFFFF`) takes his appearance from the database table
  `PlayerAppearance.bin` instead, and his boots and gloves from `BootsList.bin`/`GloveList.bin`
  (player ID, item ID pairs next to the boots/glove models). If that holds, the compiler writes
  those three tables into its CPK the way it writes `UniColor.bin`/`TeamColor.bin`, and the
  savefile patch shrinks to names and the stats half. What the result decides: Phase 4 `plan/`
  (boots/gloves assignment output) and `bins/` (which tables are accumulated; `libs/pesdb`
  possibly moving up from Phase 19), Phase 5 savefile writing and the patch format, what an
  absent `settings.toml` key means, the Save editor's appearance, transplant and diff features,
  and Phase 6's "`settings.toml` from savefile aesthetics". Test 1 (PES 2021, team `/a/`) files
  are prepared, outside git, in `.tmp/4_0/apptest/out/` with install, undo and reading instructions
  in its `manifest.txt` and a step-by-step for the maintainer in `GUIDE.txt`. Follow-ups (field meaning in `Player.bin`, whether an in-game edit
  writes the record back, other PES versions: 2017 tables in `E:\PES2017\Data\dt10_win_files\
  common\etc\pesdb`) are planned from Test 1's result → done when: the result is recorded in the
  log and in a decision entry, and the plan changes it implies are written. Done 2026-10-03, seven
  runs on PES 2021 (the 2023 Winter Cup save and DLC; run records, scripts and every finding in
  `.tmp/4_0/apptest/results.txt`, outside git): with the save's appearance id at -1 the game takes
  the player's appearance from `PlayerAppearance.bin` (per player: Look A and Look B both showed),
  his boots from `BootsList.bin`, and with the stock exe his gloves from `GloveList.bin`, without
  needing a goalkeeper position in `Player.bin`; a kept id wins (the save's look and boots). The
  4cc gloves patch (patched exe and FoxDen) overwrites the table's gloves with the save record's
  field, so gloves need a FoxDen change ("Issues"). Any Edit-mode change to a stripped player
  restores his id: a look edit freezes the shown database look into the save (boots 0), a rename
  brings back his old save look; untouched players stay at -1. Plan changes: decision entry
  "aesthetics travel in the database tables"

- [x] 4.1 **Acceptance step** (lead): the Phase 4 scope paragraph and 126 scenarios written into
  `team_compiler/README.md` "Acceptance" (new areas `PRT`, `MOD`, `TEX`, `CHK`, `XML`, `CMN`,
  `BIN`, `PLN`, `REF`, `DEP`; `KIT`, `ROOT`, `OUT` continued), with the Phase 3 edits of TC-OUT-02,
  TC-GUI-02 and TC-SRC-01; the catalog rows `model_gltf_unsupported` and `bin_source`
  (`messages.md`); the itemization rulings written into the plans (decision entry "Phase 4
  itemization rulings"); `GLOSSARY.md` lines for teams part, placeholder CPK and output sink.
  Plan: `AGENTS.md` "Working documents" (before a tool phase). Crates: none → verify: `just
  acceptance` (report mode) lists every new ID as unproven and reports no definition error;
  `rg -c "^TC-" docs/plans/team_compiler/README.md` equals 203 (77 plus 126). Done 2026-10-03:
  `just acceptance` 203 scenarios, 77 proven, the 126 new ones unproven, no definition error;
  drafted and written by Fable from the lead's rulings (`.tmp/4_1/rulings_4_1.md`)

- [x] 4.2 **Portraits (DDS, Fox targets)**: `TaskKind::Portrait`; a player folder's
  `portrait.dds` (every slot it is mapped to) and `Portraits/player_NN.dds` emitted byte for byte
  as `common/render/symbol/player/{id}{NN}.dds` (PES 19-21) or `player_{id}{NN}.dds` (PES 18);
  still refused by the Phase 3 gate: a non-DDS portrait (encoding is 4.6's: BC3 with a full mip
  chain, `player_folders.md` "Portraits") and a slot with both sources (`portrait_conflict` is
  4.7's deep pass). The tracer's Studio fixture gains its `portrait.dds`. Plan: `pipeline.md` "2.
  Per-export serial steps" step 4, "Game paths reference" (Portraits). IDs: none cited yet
  (TC-PRT-01 and 03 complete at 4.6, TC-PRT-02 at 4.7). Crates: tc (`plan/`, `processing/`,
  `paths.rs`) → verify: the tracer parity test's `79205.dds` row is `Exact` and passes; compile
  tests show `71405.dds`/`71407.dds` on PES 21 and `player_71405.dds` on PES 18 byte-identical to
  their sources, a two-slot mapping emitting both, and the two refusals. Done 2026-10-03 (Fable,
  landed first time, 0 contradictions): parity's `79205.dds` row `Exact` and passing; four compile
  tests and four unit tests, red runs in the report (old gate `content_not_yet_compiled
  (what=Players/05 - A/portrait.dds)`; perturbations of `is_dds`, `folder_holds_portrait`, the
  sort, the version and the parity bytes each failed); gates green (50 runs, acceptance 77 of
  203); `mutants-diff a949664`: 26, 21 caught, 5 unviable, 0 missed. Lead fix: the archive
  fixtures' README (they keep the eight pre-portrait files)

- [x] 4.3 **Planned IDs and player-exclusive boots/gloves (Fox)**: `plan/ids.rs` `PlannedModelIds`
  (block start `101 + (team_id - 701) * 40`, exclusive `block_start + NN - 1`, boots and gloves
  namespaces independent; frozen in the manifest, processing never allocates: the Phase 4 entry
  gate); `TaskKind::Boots`/`Gloves` for a folder's `*_boots`, `*_glove_l`/`gloveL`, `*_glove_r`/
  `gloveR`, `handL`/`handR` models (the suffix table), prefix stripped (`kit_boots.fmdl` →
  `boots.fmdl`); `boots.skl` injected from the PES 21 `body.skl` unless a same-basename `.skl`
  pairs the model; packed into `Asset/model/character/boots/k{id}/#Win/boots.fpk` + `.fpkd`,
  `glove/g{id}/#Win/glove.fpk`; the folder's textures relocated once to its common subfolder and
  committed after its face/boots/gloves tasks (`shared_texture_conflict`, canonical winner face >
  boots > gloves). The tracer's boots and gloves join the Studio fixture; the parity test's ID
  normalization (k2180 → k3745) lands. Plan: `pipeline.md` "3. Per-model-folder parallel steps"
  steps 2, 3, 6, 7; `player_folders.md` "Assigns IDs automatically", "Model names: a free part plus
  a suffix", "SKL pairing"; `development_plan.md` "Phase 4" entry gates. IDs: TC-MOD-01..04,
  TC-PLN-01. Crates: tc (`plan/ids.rs`, `plan/manifest.rs`, `processing/model.rs`) → verify: the
  tracer parity test passes over all 17 `red/` files (IDs normalized, textures matched by
  content); a `plan/ids.rs` unit test gives `/co/` slot 05 boots 625 and gloves 625, slot 23 643.
  Done 2026-10-03 (Fable; two rework rounds, two sidekick contradictions accepted): one
  `TaskKind::Models` per package plus a folder's `Textures` task, the writer committing a player
  folder as one group sharing one memory permit (decision entry "a player folder's tasks commit as
  one group"); `ModelSuffix`/`model_suffix` public in `aesthetics_export`, one matcher for every
  model; parity passes over the 17 `red/` files. TC-MOD-01/02/03, TC-PLN-01 cited; TC-MOD-04's
  conflict half waits for 4.5's reserved subfolders. Red runs in the reports (old gate:
  `content_not_yet_compiled (what=.../boots.fmdl)`; group commit: `nothing of slot 05 is in the
  CPK`; shared permit: `Arc::ptr_eq` failed with the guard at `true`). Gates green (acceptance
  81 of 203); `mutants-diff cd2e979`: 80 caught, 15 unviable, 1 timeout (an infinite loop), 1
  missed (`compile.rs:226` group guard), now killed by the shared-permit test

- [x] 4.4 **Shared folders and links (Fox)**: shared `Boots/`/`Gloves/` folders take the 17 shared
  IDs (644 upward for `/co/`) in alphabetical folder-name order, `boots_id_pool_exhausted`/
  `gloves_id_pool_exhausted`; a link alone makes the player wear the shared ID (no per-player
  output); a link plus local parts merges the shared model into the player-exclusive folder
  (`link_combined`, `fmdl::ops::merge`, alphabetical source order, `fmdl_merged`), the shared
  folder untouched; a shared folder every linker combines emits nothing and takes no ID; a shared
  face link is the player's face FMDL (merge of one); `merge_material_conflict`,
  `skl_merge_conflict` (content hash). Plan: `player_folders.md` "Shared models", "A link plus
  local models combines", "Merge constraint"; `pipeline.md` step 3. IDs: TC-MOD-05..09, TC-PLN-02.
  Crates: tc (`plan/ids.rs`, `processing/model.rs`) → verify: a `/co/` export with `Boots/Crocs/`
  linked plainly by slots 03 and 07 and combined by slot 05 (own `kit_boots.fmdl`): the CPK holds
  `boots/k0644/` once, `boots/k0625/` whose FMDL's mesh count equals Crocs's plus the local
  model's, and no `k0623`/`k0627`. Sliced: (a) shared IDs, plain-linked shared output with its
  textures in its own `k{id}/#windx11/` (measured: Red's boots FMDL paths
  `/Assets/pes16/model/character/boots/k{id}/`; 306 of 400 VGL26 gloves FMDLs name
  `…/glove/g{id}/`), pool exhaustion in the structure pass (TC-MOD-05/06, TC-PLN-02);
  (b) combining, merges and their conflicts, shared face links (TC-MOD-07/08/09).
  (a) done 2026-10-03 (Fable, first time; contradictions accepted: the `Faces/` file refusal is
  unreachable, validation dropping every shared folder no mapped player links; lead fix:
  `SharedModelFolder.path`): `PlannedModelIds::shared`, `ids::shared_folders_taking_ids`
  (case-folded order), `paths::TextureHome`, `plan::ModelFolder` for player and shared folders
  alike, `structure::pool_messages`; decision entry "shared boots/gloves outputs". TC-MOD-05/06,
  TC-PLN-02 cited. Gates green (84 of 203); `mutants-diff ccd9173`: 40, 34 caught, 6 unviable,
  0 missed. (b) split into b1 (boots/gloves) and b2 (faces, texture byte conflicts). (b1) done
  2026-10-03 (Fable, first time; contradictions accepted: a stem two *combined* folders hold
  collides too, refused for any two of the player's sources; a plainly linked shared folder's
  parts merge as well. Lead fix: stems compared case-folded): `ModelFolder.combined`/`sources()`,
  `subset::link_combines` shared by gate, IDs and planning, `processing::TaskFailure` carrying
  `merge_material_conflict`/`skl_merge_conflict`, `FolderModels.boots_stems`. TC-MOD-07/09 cited.
  Gates green (86 of 203); `mutants-diff b744ff3`: 43, 36 caught, 7 unviable, 0 missed. Not
  verified: the game loading a merged FMDL (`Model::to_file` re-lays it out). (b2) done
  2026-10-03 (Fable, first time, no contradiction; decision entry "texture conflicts by source
  package"): face links combine, face parts merge (`fcl_hair_sim.skl` by the boots rule),
  `ModelFolder::roles()` with the player's own `face_diff.bin`/`.fclo` winning,
  `processing/texture.rs` resolving a stem per source package (`shared_texture_conflict` drops the
  lower package via `TaskBatch.skipped`, `merged_texture_conflict` fails the textures task).
  TC-MOD-08 cited; TC-MOD-04's halves tested uncited (its GIVEN needs 4.5's subfolders). Gates
  green (87 of 203); `mutants-diff 6bd79c9`: 34, 28 caught, 6 unviable, 0 missed. Texture-name
  case questions in "Issues"

- [x] 4.5 **Face assembly (Fox)**: unsuffixed models routed to the `fcl_hair` merge (`torso.fmdl` +
  `legs.fmdl.common` → `fcl_hair.fmdl`; `fmdl_fcl_hair_fallback` per routed file; same-name
  merges themselves landed at 4.4b, `processing/model.rs`), `.common` model
  links baked in with their textures referenced in place in the team's Common output
  (`Asset/model/character/common/{team_id}/sourceimages/#windx11/`), `merged_texture_conflict`;
  reserved subfolders `face/`/`boots/`/`gloves/`/`common/` force the category; a custom `.skl`
  paired with the `fcl_hair` part replaces the injected `fcl_hair_sim.skl`, `skl_no_slot` for
  `face_high`/`hair_high`/`oral`; `face_diff.bin` and `fcl_hair_sim.fclo` injected from templates
  (lead first: the two templates as lead-authored fixtures with a provenance README,
  `resources/bins/README.md` the model; the tracer's copies are byte-identical to Red's), so a
  face folder lacking them compiles, which Phase 3 refused; `face_diff.xml` → `face_diff.bin`
  (`face_diff_invalid`). Plan: `pipeline.md` step 3 "Fox mode fixups", step 6;
  `player_folders.md` "Common model links and model merging", "Reserved subfolders", "What the
  injected files are"; `model_format.md` "Link files". IDs: TC-MOD-10..15. Crates: tc
  (`processing/model.rs`, `templates.rs`), resources → verify: a `/co/` player with `torso.fmdl`,
  `legs.fmdl.common` and `Common/legs.fmdl` referencing `Common/cloth.dds`: the face package holds
  one `fcl_hair.fmdl` whose mesh count is the sum, `cloth.ftex` sits once under
  `common/714/sourceimages/#windx11/` and is absent from the player's common subfolder, and the
  console shows `fmdl_fcl_hair_fallback` then `fmdl_merged`. Sliced: (a) templates, fallback,
  `skl_no_slot`; (b) reserved subfolders; (c) `.common` model links; (d) `face_diff.xml`, decoded with
  the `base64` crate (decision entry 2026-10-03). (a) done
  2026-10-03 (Fable, first time; lead first: `resources/templates/` from Red with a provenance
  README): `templates::{FACE_DIFF, FCL_HAIR_SIM_FCLO, BODY_SKELETON}` injected per Face package,
  unsuffixed models are `fcl_hair` parts, `PlayerFile::SlotlessSkeleton`,
  `structure::model_name_messages` (findings on the folder holding the file, a shared face's
  included); the gate's `missing` checks gone. TC-MOD-12/13 cited. Gates green (89 of 203);
  `mutants-diff 5eeb04c`: 38, 33 caught, 5 unviable, 0 missed. Found: the tracer's
  `fcl_hair.skl` is PES 21's `body.skl`, so two older processing tests' "custom" skeleton equals
  the template (fix in 4.5b's brief). (b) done 2026-10-03 (Fable, first time; no contradiction
  of substance): `subset::position`/`model_role` force a reserved subfolder's category, skeletons
  pair by path stem (same directory), the two weak skeleton tests now use PES 19's `body.skl`.
  TC-MOD-14 cited, TC-MOD-04 re-scoped and cited (decision entry). Gates green (91 of 203);
  `mutants-diff 428ef6c`: 40, 34 caught, 6 unviable, 0 missed. For converge: the four reserved
  names are matched in both `aesthetics_export` (`validate::folders::position`, `pub(crate)`) and
  `team_compiler` (`subset::position`); one exported matcher would remove the copy. (c) done
  2026-10-03 (Fable; one contradiction accepted: with paths rewritten per part, TC-MOD-09's two
  `shirt` materials differing only by source directory became one, so its fixture now differs
  by shader): `PlayerFile::CommonModel`, `plan::CommonModel` resolved at planning, the Common
  `.skl` travelling with a slotted link, texture paths rewritten per part before the merge, the
  Common textures task (`TaskKind::CommonTextures`, decision entry); lead first: `aesthetics_export`
  exports `common_link_name`. TC-MOD-10/11 cited. Gates green (93 of 203); `mutants-diff
  624825f`: 86, 72 caught, 13 unviable, 1 missed (`model.rs` `CommonModel` package guard), given a
  test in one rework round (red shown). For converge: `subset::is_direct_common_file` copies
  validation's private check. (d) done 2026-10-03 (Opus 5.5, first time; two contradictions
  accepted: the catalog holds no text templates yet, and `base64` is taken without its default
  SIMD feature; lead first: the plan's `face_diff.xml` paragraph, the fixtures under
  `tests/fixtures/face_diff/` from two real bins): `processing/face_diff.rs` (`check`,
  `from_xml`), `PlayerFile::FaceDiffXml` routed wherever `face_diff.bin` is, the xml standing
  for the bin across sources and both kept within one, `face_diff_invalid` and
  `xml_dif_conflict` as failures of the Face task (the folder's boots, gloves and textures
  still commit), a supplied `face_diff.bin` checked too. A face diff passes when it is at
  least as long as its header gives: Red's exact-length check would refuse 326 of the
  machine's 2,695 loose bins (decision entry). TC-MOD-15 cited. Gates green (100 of 203);
  `mutants-diff a75e3a9`: 57, 51 caught, 6 unviable, 0 missed. For 4.7: both findings are
  reported only by `compile` until the deep pass runs the same check at `check`

- [x] 4.6 **Textures, all formats (Fox)**: every accepted image format (`dds_convert::decode`/
  `convert`) for player, shared, Common, kit and portrait textures; BC7 kept and encoded on PES
  19-21, BC7 transcoded to BC3 and raster encoded to BC3/BC1 on PES 18; missing mips generated;
  FTEX headers per version; `texture_too_small`, `texture_not_pow2`, `texture_type_mismatch`,
  `texture_codec_unsupported`, `fmdl_no_texture_ids`; `kit_texture_not_used` for a `kit_mask` on a
  Fox target (Phase 3's silent drop was a stopgap, not a decision); the `Converter` cache engaged
  for runs of at most two teams; conversion in bounded batches on a cancel token
  (`MemoryBudget::cancel`). Plan: `pipeline.md` step 5; `libs/dds_convert.md` "In-memory
  conversion cache" (engagement); `messages.md` "Textures"; `development_plan.md` "Phase 4"
  (CPU-only, GPU is 16.x). IDs: TC-TEX-01, TC-TEX-02, TC-TEX-04, TC-TEX-06, TC-KIT-10,
  TC-PRT-01 (non-DDS portraits encoded here); TC-TEX-03, TC-TEX-05 and TC-PRT-03 say "checked",
  so they are 4.7's: the deep pass parses textures and FMDLs (`pipeline.md` "2. Per-export serial
  steps" step 2), while this step raises the texture codes at conversion, failing the task.
  Sliced: (a) conversion core: `dds_convert` for every model-folder, Common and kit texture, the
  codec by version, the role by stem (`_nrm`, decision entry), the run's `Converter` and its
  team-count policy, the gate opened to every accepted format (TC-TEX-01, TC-TEX-02, the kit half
  of TC-TEX-06); (b) the conversion findings as task failures (`texture_too_small`,
  `texture_not_pow2`, `texture_type_mismatch`, `texture_codec_unsupported`; TC-TEX-04),
  `kit_texture_not_used` (TC-KIT-10), portraits encoded to BC3 DDS (TC-PRT-01, TC-TEX-06's
  portrait half; `dds_convert` has no DDS output for a Fox target yet, lead first); (c) bounded
  batches on the cancel token. (a) done 2026-10-03 (Fable; contradictions applied: the role
  computed inside `texture::convert`, `texture_format` kept returning `SourceFormat`): every
  model-folder, Common and kit texture through the run's `Converter` (`CompileContext::new`,
  `cache_policy`), the gate open to every accepted format but portraits; the BC7 encoder builds
  optimized in dev (decision entry: TC-TEX-01 took 56 s unoptimized, 9 s now). TC-TEX-01/02
  cited. Gates green (95 of 203); `mutants-diff bd3bb97`: 23, 15 caught, 8 unviable, 0 missed.
  For converge: `Converter::convert` returns `Arc<[u8]>` even under `Bypass` and `Entry` owns a
  `Vec`, so each texture is copied once (`Use`) or twice (`Bypass`); an `.ftex` source is now
  rebuilt rather than passed through. (b) done 2026-10-03 (Fable; contradictions accepted: a
  failed portrait task reports `DropFile`; the size checks read the written FTEX's header, so
  no second decode; `kit_texture_not_used` is reported before the gate, noisy until 4.20; lead
  first: the catalog's drop scopes and signature rule, decision entry, and the fixtures):
  `dds_convert::encode_dds` (sharing `convert`'s steps), `texture::TextureError`, the four
  findings, Common's `DropFile`, portraits in any format. TC-TEX-04/06, TC-PRT-01, TC-KIT-10
  cited. Gates green (99 of 203); `mutants-diff 0d26507`: 75, 68 caught, 7 unviable, 1 missed
  (`needs_pow2`'s `mipmaps > 1`), given a test with the `single_level.dds` fixture in one rework
  round (red shown); the remote half peaked at 7.90 GiB of 8G (maintainer asked). Found:
  `texture_codec_unsupported` also covers `dds_convert`'s other `Unsupported` refusals (a 16-bit
  TGA, the 4 GiB limit), which the help names. (c) moved to Phase 8's cancellation 2026-10-03
  (decision entry): the bound already holds, one texture at a time per worker; nothing in Phase 4
  cancels a run, so the between-textures check is built with the Cancel that triggers it.
  Crates: tc (`processing/texture.rs`,
  `processing/kit.rs`) → verify: a `/co/` player with `skin.png` (1024x1024, alpha) compiled for
  PES 21 then PES 18: `ftex::info` of the emitted texture reports BC7 then BC3, each with an
  11-level mip chain; a 3x3 `skin.png` reports `Error texture_too_small [DropFolder]` and the
  folder is absent from the CPK

- [x] 4.7 **Deep validation pass**, in slices (a) to (f), done 2026-10-04. The shape (decision entry "the deep pass:
  the compiler checks, the export crate drops"): tc reads the files of the sanitized export
  and runs the checks (`deep.rs`, new); `ae` takes what they find
  (`ValidationReport::with_content_findings`) and derives the sanitized export again, so the
  drops, the link cascade and `pass_through` are the structure pass's own; the pass runs
  before identity and planning, in `check` and in `compile`, on every source kind, archives
  included. A format finding maps by severity (`messages.md` "What makes a finding an Error":
  Error drops the folder, Warning and Info keep it); the lead's `fmdl::check` census over the
  VGL26 corpus (`C:\Data\4cc\Lab\Gud`) found one Error class, the far vertex, which stands.
  Plan: `pipeline.md` "2. Per-export serial steps" step 2 (deep format pass);
  `object_model.md` "Validation semantics" (content findings); `messages.md`
  (`vertex_too_far_from_origin`). IDs: TC-CHK-01..05, TC-PRT-02, and from 4.6 TC-TEX-03 and
  TC-PRT-03 (texture findings reported by `check`; TC-TEX-05 is 4.29's). Crates: ae
  (`validate/`), tc (`deep.rs`, `structure.rs`, `check.rs`, `messages.rs`, `processing/`),
  fmdl, pes_model, dds_convert (the code lists and the header probe)
  → verify: `check` on a `/co/` export whose `boots.fmdl` holds a vertex 6000 units out prints
  `Error vertex_too_far_from_origin [DropFolder]` naming the file and exits 1; `compile` with
  `pass_through` on still leaves the folder out of the CPK
  - (a) the thin path end to end: `ContentFinding` and `with_content_findings` in `ae`;
    `deep.rs` reading every `.fmdl` of the player folders, the shared folders and `Common/`
    and reporting `vertex_too_far_from_origin` (`DropFolder`, not pass-through-eligible) for
    `fmdl_vertex_far_from_origin`; TC-CHK-01..02 (lead first: the far-vertex model and the
    solid `.7z` holding it, `tests/fixtures/deep/`). Done 2026-10-03 (Opus 5.5, first
    time): `ContentFinding`, `ValidationReport::with_content_findings` (`validate`'s one
    body, the findings standing before the cascade; kits, portraits, collars, the logo and
    a player's `settings.toml`/portrait now read the dropped scopes too); tc `deep.rs`,
    `structure.rs` renamed `validation.rs` (`validation_pass`), the help topic. Gates green
    (102 of 203); `mutants-diff f937e3d`: 29, 16 caught, 12 unviable, 1 missed
    (`is_fox_model -> true`: a file that does not parse has no finding yet, so reading
    every file as a model changes nothing; (b)'s `model_broken` makes it observable and
    its brief asks for the test). Not covered, for (c)/(d) if they need it: a drop naming
    `Kits/all`, a kit's `config.toml` or `colors.txt`, or the root `colors.txt`
  - (b) every format finding by its severity. Done 2026-10-03 (Opus 5.5, one rework
    round): the deep pass reads every `.fmdl`, `.model` and `.mtl` and reports each check
    code of `fmdl` and `pes_model` once per file with its summed count (`deep.rs`); an Error
    drops the folder and is pass-through-eligible, the far vertex and a file that does not
    parse (`model_broken`, `mtl_broken`) are not; `CODES` in both crates' `check.rs` is the
    one place a severity is written, and the catalog test holds every format code's row to
    it; a rule about a mesh, material or bone itself now counts 1, not 0. TC-CHK-06..07
    (new). The tracer's models carry `fmdl_weights_not_normalized` (Info), so most CLI
    tests gained those lines; placeholder models in tests are now a real clean one. Gates
    green (104 of 205); `mutants-diff eb5fb67`: 96, 83 caught, 13 unviable, 0 missed. `check_bundle`
    (`model_material_undefined`) is the pre-Fox face steps'
  - (c1) textures of model folders, `Common/` and kits from their headers
    (`dds_convert::probe`, no decode): `texture_type_mismatch`, `texture_too_small`,
    `texture_not_pow2`, `kit_texture_too_big`; processing's own checks on those sources
    retired, since the deep pass has run them and `pass_through` must be able to keep the
    file (`texture_codec_unsupported` stays conversion's). Plan: `messages.md` "Textures",
    `libs/dds_convert.md` "`dds_convert` API", decision entry "texture checks: from the
    header, in the deep pass". TC-TEX-03, TC-CHK-04. Done 2026-10-03 (Opus 5.5, one
    tests-only rework round): `dds_convert::probe` (`Probe`; a raster source's level count
    from `mips::chain_len`, the one place the chain's length is written); in `deep.rs`,
    `Checked::Texture(SourceFormat, SizeRule)`, `texture_finding` (the first rule that
    fires: mismatch, too small, then the kit's main texture or the Fox mip rule) and the
    signature table, moved from processing; a kit's findings on the kit's folder, an
    `all/` texture read once and named by its export path on each inheriting kit;
    `texture::convert` checks nothing itself any more (a portrait's checks stay in
    processing until (c2)); an unreadable texture is `source_read_failed` like a model.
    Under `pass_through` a 3x3 texture converts and is packed. Gates green (106 of 206);
    `mutants-diff e6e9f21`: 123, 73 caught, 1 timeout (an infinite loop), 42 unviable, 7
    missed, all the size rules asked about square textures only; after the rework's table
    test: 123, 80 caught, 1 timeout, 42 unviable, 0 missed. `deep.rs` is 1,258 lines with its tests: (c2) splits it into `deep/`
  - (c2) portraits from their headers (the file dropped; a power-of-two side always),
    `portrait_conflict` (both sources, different bytes: the export dropped; equal bytes:
    one portrait, and planning's refusal of a slot with both goes), the logo's decode
    (`logo_file_invalid`), processing's portrait checks retired. TC-PRT-02..03, TC-CHK-05
    (its "no logo is emitted" half is trivially true until 4.11; the test asserts the
    CPK's exact entries, so it keeps proving it afterwards). Done 2026-10-03 (Opus 5.5,
    first time): `deep.rs` split into `deep/mod.rs` (the walk), `deep/model.rs` and
    `deep/texture.rs`, a pure move; a portrait checked on its own `File` scope
    (`SizeRule::Portrait`), a player's after its folder's files, the `Portraits/` files
    after `Common/`; `portrait_conflict` (`Export`, `DropExport`) compares the bytes of a
    slot's two files whatever their own findings (each is read a second time for it);
    planning keeps the folder's file for a player id with both, and the gate's refusal is
    gone; the logo's files decoded last (`Checked::Logo`); `portrait` in processing still
    decodes every source, so a broken one fails its task, and checks nothing else. Only
    team rosters pair a folder's portrait with a `Portraits/` file. Not tested in the
    deep pass: a folder two slots map, paired with each slot's file (planning's side is).
    Gates green (109 of 206); `mutants-diff 8eb7931`: 65, 52 caught, 13 unviable, 0 missed
  - (d) `face_diff_invalid` and `xml_dif_conflict` reported by the deep pass, where they drop
    the folder (`player_folders.md` "`face_diff.xml`"); `kit_config_invalid`,
    `settings_toml_invalid` (parse only, through `pes_savefile`'s `PlayerSettings` TOML
    reader; the other `settings_*` and `fpc_strip_conflict` codes describe resolved values
    and are Phase 5's). TC-MOD-15, TC-CHK-03. Done 2026-10-03 (Opus 5.5, first time; three
    lead fixes): `deep/documents.rs` (the face diff of each player folder and shared face
    folder, over the files planning's `player_file` gives a face diff's role; a kit's
    `config.toml` before its textures; a player's `settings.toml` after its portrait, on
    its own `File` scope); the portrait checks moved to `deep/portrait.rs`, a pure move;
    `face_diff.rs` now at the crate root; `pes_savefile` is a dependency of tc. Processing
    lost its conflict check and its `face_diff.bin` check, and a `face_diff.xml` that
    fails to decode there is an ordinary `folder_pack_failed` (unreachable from the CLI,
    so untested); planning's `roles` no longer keeps both forms of one source (lead fix,
    with the duplicated test helper and a doc comment). A face diff finding now names its
    file below the folder (`face_diff.xml`, not the export path). Not covered: a folder
    whose only face models are `.model` files gives its face diff no role, so it is
    unchecked until the pre-Fox face steps. Gates green (110 of 209);
    `mutants-diff 90e5bfb`: 38, 25 caught, 13 unviable, 0 missed
  - The timing, done 2026-10-03 (lead; release build of `78fc7c2`, 16 logical CPUs, warm
    file cache, median of 3; `.tmp/4_7/timing_build.py` lays an old-layout VGL26 export out as
    `Players/NN - Name/`, `.tmp/4_7/timing_run.py` times it). FNG, the corpus's largest: 991 MB,
    245 files, 880 MB of DDS (nine 8192x8192 textures of 67 to 90 MB), 111 MB of models;
    its solid `.7z` is 78 MiB. DBG: 604 MB, 214 files, 355 MB of models, 249 MB of DDS; its
    `.7z` is 111 MiB.

    | seconds | `check` folder | `check` `.7z` | `compile` folder | `compile` `.7z` |
    |---|---|---|---|---|
    | FNG | 0.55 | 2.59 | 3.08 | 7.19 |
    | DBG | 0.82 | 2.76 | 1.74 | 5.78 |

    `check` on a folder is nearly all deep pass, so the serial pass is 18% (FNG) to 47%
    (DBG) of a folder compile. Reading every file whole takes 0.41 to 0.50 s on FNG and
    0.19 s on DBG (`.tmp/4_7/timing_read.py`): FNG's pass is whole-file reads of textures for
    their headers, DBG's is model parsing. Each `.7z` decompression costs about 2 s, and a
    compile of these exports does two, about 4 s of its 5.8 to 7.2 s (corrected at (f): first
    written as three of 1.0 to 1.4 s, the count assumed; the timing exports hold no metadata
    file, so routing never decompressed them). Not measured: a cold cache, a run of
    many exports. Found: every face folder of both exports holds only a `face_diff.bin` and
    a portrait (an in-game face with a diff), which `compile` refuses today as
    `content_not_yet_compiled` (4.12's); the timing exports leave those files out
  - (e) the pass on the worker pool (decision entry "the deep pass runs on the worker pool,
    within one export"; `pipeline.md` "Deep format pass"): an export's model folders and
    `Common/` files checked in parallel, a folder's files too, findings in file order;
    `ContentSource::read` on `&self`, an archive behind a lock; the sources routed in
    parallel; folder and `.zip` exports checked in parallel with each other, `.7z` ones in
    turn (decision entry "folder and zip exports are checked in parallel");
    `check` and `compile` share one pool. Done 2026-10-03 (Opus 5.5, first time; resumed
    once after an IDE restart; one lead fix, a test comment): `validation::run_pool`,
    built at the start of both commands; `ContentSource::read(&self)`, the archive and its
    permit in one `Mutex` held for one read; `par_iter` with an indexed `collect` over
    the player folders, the shared folders, a folder's files and `Common/`'s files, the
    portraits, kits and logo in order; `check_sources` (folders and zips in parallel,
    then the `.7z` ones, merged by discovery index); `reader::route` in parallel. No
    existing test changed. The brief's premise that one worker thread would expose a
    lock held across parallel work was wrong (the sidekick showed it does not hang): the
    guard is the read's shape, owned bytes with the lock released. Gates green (110 of
    209); `mutants-diff 828242e`: 21, 12 caught, 9 unviable, 0 missed. Measured again
    (same setup, median of 3, before → after):

    | seconds | `check` folder | `check` `.7z` | `compile` folder | `compile` `.7z` |
    |---|---|---|---|---|
    | FNG | 0.55 → 0.32 | 2.59 → 2.45 | 3.08 → 2.87 | 7.19 → 7.01 |
    | DBG | 0.82 → 0.34 | 2.76 → 2.35 | 1.74 → 1.30 | 5.78 → 5.32 |
    | both in one run | 1.31 → 0.55 | 5.05 → 4.72 | 4.69 → 3.95 | 12.86 → 12.03 |

    What is left of a folder compile is planning, the coordinator's reads, the tasks and
    the CPK's writing (FNG: about 2.5 s of 2.87); of a `.7z` compile, mostly its
    decompressions (two for these exports, three for one holding a metadata file: (f))
    → verify: the timing table above measured again, every existing test unchanged
  - (f) a `.7z` decompressed once for the structure pass and the deep pass (decision entry
    "a source's metadata is read by its check, not by routing"; `libs/pipeline.md` "What a
    solid `.7z` is charged"). Done 2026-10-04 (Opus 5.5, first time, no lead fix): routing
    only lists (`reader::source::list`; `route` takes no budget, `Route::Validate` holds the
    listing alone); `ContentSource::read_metadata`, called by `check_source` on the handle
    the deep pass then reads from (`deep::content_findings` takes the `ContentSource`),
    dropped when the deep pass ends; `OpenSource` and its archive-only metadata read are
    gone, one read path for the three source kinds. A disabled, balls or conflicting-refs
    `.7z` is never decompressed. No existing test's expectation changed; one new test (the
    metadata read and the later reads share one permit, held until the source is dropped).
    Gates green (110 of 209); `mutants-diff 3ea4018`: 11, 4 caught, 7 unviable, 0 missed.
    Measured (release build, same setup, median of 3, the 4.7e binary → this one). The
    timing exports hold no metadata file, so routing never decompressed them and they
    measure the same as before (FNG `.7z`: `check` 2.17 → 2.28, `compile` 6.36 → 6.62,
    within the runs' spread). With a root `notes.txt` added to each
    (`.tmp/4_7/timing_notes.py`):

    | seconds, `.7z` with `notes.txt` | `check` | `compile` |
    |---|---|---|
    | FNG | 4.39 → 2.36 | 8.82 → 6.79 |
    | DBG | 4.23 → 2.31 | 7.19 → 5.21 |
    | both in one run | 6.95 → 4.77 | 14.02 → 11.76 |

    A decompression costs about 2 s per export (both in one run save 2.2 s, not 4: routing
    decompressed the two in parallel). The folder runs are unchanged (FNG `check` 0.32 →
    0.29, `compile` 2.74 → 2.73). Left: the tasks' decompression, about 2 s of a `.7z`
    compile per export ("Issues")

- [x] 4.8 **Kit colors, UniColor and TeamColor (the `bins/` module)**: kit `colors.txt` grammar
  (`player_folders.md` "Root files", "Colors": one color per line in both files, the
  maintainer's ruling of 2026-10-03), `color_entry_invalid`, derivation via
  `color_tools::kit::extract_kit_colors` on the decoded main texture (`kit_colors_derived`), the
  magenta/black pair (`kit_colors_missing`), the icon marker (`icon_<N>`, step 4.30; default
  3); `UniColor.bin` built on the
  bundled base (lead first, done 2026-10-03: `resources/bins/TeamColor.bin` and
  `UniColor.bin`, Red's, with the measured record layout in the folder's README and
  `scripts/provenance/fixtures/color_bins_compare.py`) at Red's per-team offsets, an entry applied only when its
  kit's task commits (placeholder kits included: TC-SRC-01's p2 gains its entry); root
  `colors.txt` → `TeamColor.bin`, `team_colors_missing` (I); both bins written whole with
  every record's header set from its position, `bin_header_repaired` (W) when one was wrong
  (TC-BIN-13; installed bins carry headerless `TeamColor.bin` records). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Bins accumulation), "Resolved decisions" (Kit colors fallback);
  `libs/color_tools.md` "Dominant kit-color extraction". IDs: TC-KIT-11..14, TC-ROOT-10,
  TC-BIN-01..03, TC-BIN-13. Crates: tc (`bins/mod.rs`, `processing/kit.rs`), resources (`resources/bins/`) →
  verify: a `/co/` export with `p1/colors.txt`, `p2/kit.dds` without colors and an empty `p3/`:
  the CPK's `UniColor.bin` carries at team 714's p1 offset the file's two colors, at p2's the
  pair `extract_kit_colors` returns for the decoded texture, at p3's the magenta/black pair; the
  tracer parity test compares `TeamColor.bin` and `UniColor.bin` byte-identical (the tracer's
  Studio fixture gains its root and `g1` `colors.txt`, the note's colors; lead first). In
  slices (decision entry "kit colors merge into a team's UniColor record by kit number"):
  - (a) the `colors.txt` reader (`aesthetics_export::colors_txt`) and `color_entry_invalid`
    from the deep pass, for the kit files and the root file, so `check` reports it. Done
    2026-10-04 (Opus 5.5, first time, no lead fix): `read_colors_txt(bytes, capacity)`
    giving the valid colors and the refused lines (`NotOneColor`, `PastCapacity`), each
    line decoded alone; `deep/documents.rs` `colors_findings`, one Warning per refused line
    on the file, kept, with `line` and `reason`, a kit's after its config and the root
    file's last; the help paragraph. The reader also settles: separators are spaces and
    commas only (a tab is refused), a leading `+` is refused, leading zeros are read.
    Open for (b): a `colors.txt` that cannot be read is reported `DropFile` but stays in
    the validated export. Gates green (110 of 209); `mutants-diff a9d9a51`: 37, 34 caught,
    3 unviable, 0 missed
  - (b) `TeamColor.bin`: `bins/` (the bin as records, every header set from its position,
    `bin_header_repaired`, the bundled base as the run's working bin); the root `colors.txt`
    read while the export's source is open and applied for each export planning keeps;
    `team_colors_missing`; the writer adds the bin to every CPK it writes; a `colors.txt`
    the deep pass drops leaves the validated export. TC-ROOT-10; the tracer parity row
    `TeamColor.bin` exact. TC-BIN-13's repair is tested at the writer on a working bin
    given to it; its CLI proof needs an installed bin, so the ID's citing test lands with
    4.21. Done 2026-10-04 (Opus 5.5, first time; two lead fixes: a root file whose second
    read fails gives no color and is not reported missing, and one test for a mutation
    survivor): `bins/mod.rs` (`TeamColorBin`: `read`, `repair_headers`, `set_colors`;
    `WorkingBins::bundled`, the seam 4.21 fills from the installed CPKs; `Rgb`, the two
    capacities); `validation::team_colors` reads the root file after the deep pass, a
    referee export's never; `plan_run` takes each export's colors and lists them in
    `BuildManifest::team_colors` for the exports the gate keeps, reporting
    `team_colors_missing` (compile only, after the gate's findings);
    `CpkOutput::finish(version, bins, team_colors)` adds `common/etc/TeamColor.bin` after
    `UniformParameter.bin` whenever a CPK is written and returns `bin_header_repaired`
    for `compile::run` to report; `validate_with` filters `KitFolder::colors` and
    `team_colors` by `file_kept`. The tracer's `TeamColor.bin` equals Red's byte for byte.
    69 CLI tests and one of `studio` gained the bin's entry or the `team_colors_missing`
    line, nothing else. Not covered: a `.7z` export with a root `colors.txt` (no fixture);
    `bin_header_repaired` from the CLI (4.21). Gates green (111 of 209);
    `mutants-diff b685288`: 37, 27 caught, 9 unviable, 1 missed (`read`'s record-count
    limit), closed by a test seen failing on the mutant
  - (c) `UniColor.bin`: the kit task's colors (its `colors.txt`, else derived, else the
    magenta/black pair) and icon, merged into the team's record by kit number; TC-KIT-11..14,
    TC-BIN-01..03; the tracer parity row `UniColor.bin` exact. Done 2026-10-04 (Opus 5.5,
    first time; two lead fixes, a help sentence and a test local's name): `bins/mod.rs`
    (`Records`, the plumbing both bins share: length check, a team's record, the header
    loop; `UniColorBin`: `read`, `repair_headers`, `set_kit`; `KitColorEntry`,
    `kit_number`; `WorkingBins::uni_color`); the kit task reads `colors.txt` and builds
    the entry (`processing/kit.rs`: `listed_colors`, `derived_colors`, `MISSING_COLORS`
    magenta then black, `DEFAULT_ICON` 3), reporting `kit_colors_derived` (I) or
    `kit_colors_missing` (W) on the kit folder; `TaskBatch::uni_color` travels beside
    `uniparam` and the writer applies it only for a batch that commits;
    `CpkOutput::finish` adds the bin after `TeamColor.bin` on every CPK and returns a
    second `bin_header_repaired` for it. Colors derive from the `kit` texture of the
    kit's effective set, one inherited from `all/` included (decision entry). The
    tracer's `UniColor.bin` equals Red's byte for byte. TC-KIT-14's GIVEN changed (a
    `kit.dds` no decoder reads: the deep pass refuses a non-UTF-8 `config.toml` before
    any task runs). 42 existing tests gained the bin's entry or a `kit_colors_*` line,
    nothing else. The derivation decodes the main texture once more than its conversion
    does (`dds_convert` shares no decode): measured on three real 2048 kits (release
    build, median of 5), a compile takes 0.167 s without `colors.txt` files against
    0.140 s with them (DXT1), 0.259 against 0.216 s (DXT5). Not covered: a main texture
    whose shirt region is fully transparent (magenta and black, by reading); the colors
    a raster (`.png`) main texture gives. Gates green (118 of 209); `mutants-diff
    ccbd7c4`: 77, 59 caught, 18 unviable, 0 missed
  The tracer's Studio fixture holds both `colors.txt` files since 2026-10-04 (lead)

- [x] 4.9 **Kit configs, FPC reconciliation and collars** (a collar in the other engine's
  format is converted, decision 2026-10-05; `Collars/` admits any model format):
  team kit-FPC status (`fpc_on` in any
  player folder → On), `fpc::kit_values` returning the four FPC values on every version,
  PES 15 and 18 included, so without its `Option` (both are `None` today, from a wiki page
  silent on them; PES 15 measured on the install, PES 18 confirmed by the maintainer;
  `libs/fpc.md` has the counts), with `kit_config`'s `apply_fpc`/`matches_fpc` simplified to
  match,
  generated configs with FPC values (`kit_config::fpc::apply_fpc`), supplied
  configs reconciled upward (`kit_config_fpc_adjusted`, GK included),
  `kit_config_version_clamped`; `fpc_on`/`fpc_off` no longer refuse the export (their savefile
  half is Phase 5); `Collars/` gets its allowlist row in `ae` (model files named `collar_<ID>`,
  any model format; the per-version stock sets are in `messages.md` `collar_id_invalid`,
  measured on every install, PES 20's included since 2026-10-03), `collar_<ID>` parsed with or
  without zero padding, `collar_id_invalid` (not a stock collar of the target version),
  `kit_collar_reserved` for a regular team's kit
  whose effective collar or winter collar is 77 (kit dropped; `blue_port.md` "Referee export
  processing", TC-REF-07),
  `collar_id_conflict` in canonical export order, the claimed-ID list starting with the
  reserved 105 (FPC) and 77 (the referees' marker), every kit config's collar fields rewritten
  after FPC, collar models converted to the target's format at its `nocloth` path under the
  game's three-digit name. Plan: `fpc_toggle.md` "Team
  kit-FPC status and kit configs"; `pipeline.md` "4. Per-export non-model steps" (Kits, Collars),
  "Resolved decisions" (Collar contract); `object_model.md` "File-type allowlist". IDs:
  TC-KIT-15..17, TC-CMN-01..03. Crates: ae (`conventions/file_types.rs`, `validate/folders.rs`),
  tc (`processing/kit.rs`, `processing/team_assets.rs`, `plan/`) → verify: a `/co/` export with
  `fpc_on` in slot 05 and a supplied `p1/config.toml` without FPC values: the emitted 120-byte
  config decodes with `kit_config::fpc::matches_fpc` true and `kit_config_fpc_adjusted` is
  reported for p1; with `Collars/collar_12.fmdl` added, every emitted config's collar fields read
  12 and the file sits at `Asset/model/character/uniform/nocloth/#Win/collar_012.fmdl`.
  In slices (decision entry "4.9a: the FPC kit values take no version; the template carries
  them"):
  - (a) FPC reconciliation and the clamp warning: `fpc::kit_values()` with no version and
    `u8` values, `kit_config`'s `apply_fpc`/`matches_fpc` to match; the team's kit-FPC status
    from the validated export's player folders; a supplied config lacking the FPC values
    gets them when the status is On (`kit_config_fpc_adjusted`); `fpc_on` no longer refuses
    the export; `kit_config_version_clamped` from the deep pass. TC-KIT-15..17. Done
    2026-10-04 (Opus 5.5, first time, no lead code fix): `fpc::kit_values()`;
    `kit_config::apply_fpc(&mut config)` and `matches_fpc(&config)`;
    `plan::EffectiveTeamKitFpc::of(&export)`, carried by `TaskKind::Kit { fpc }`; the kit
    task (`processing/kit.rs`) applies the values to a config lacking them when the
    status is On and reports `kit_config_fpc_adjusted` (I, kit folder, kept), and never
    reverts; `plan/subset.rs` no longer names `fpc_on` as content not yet compiled;
    `deep/documents.rs` `kit_config_findings` runs `kit_config::validate` on a config
    that parses and reports `kit_value_out_of_range` and `kit_pattern_unsupported_pes15`
    as `kit_config_version_clamped` (W, on the file, kept; `field`, `value`, `max`), so
    `check` prints it too (`toml_finding` became `parsed_toml`, returning the parsed
    value); two help paragraphs. TC-OUT-06's and TC-OUT-02's tests now use an
    `ingame_face` export as their not-yet-compiled example. Found: the template's
    `name.y` is 30, over PES 15 to 20's 16 ("Phase 4 open questions"). Not covered: the
    PES 15 pattern clamp from the CLI (the deep pass's unit test has it; `compile`
    refuses PES 15); a goalkeeper config adjusted at the CLI (the task's unit tests use
    `g1`). Gates green (121 of 209); `mutants-diff f31075e`: 34, 24 caught, 10 unviable,
    0 missed
  - (b) collars (`Collars/`'s allowlist row, `collar_id_invalid`, `collar_id_conflict`,
    `kit_collar_reserved`, the configs' collar fields rewritten after FPC, the models converted
    to the target's format, decision 2026-10-05; models only, drawn with the kit texture).
    TC-CMN-01..03, TC-CMN-08..10. A collar converted to Fox keeps its converted materials
    (Fox embeds them in the FMDL); one converted to pre-Fox gets `uni_collar`/`uni_shirts`
    (`pipeline.md` "Collars"). `kit_collar_reserved` landed with 4.27a. Plan settled
    2026-10-07 (decision entry "collars: the 9xx IDs, one collar per export, a midcup's absent
    slots"). Slices:
    - [x] 4.9b1 the checks: done 2026-10-07 (one rework round: the lead's brief had
      exempted collars from the model checks on a wrong premise; the sidekick's
      contradiction, accepted). `ae` `folders::check_collars` (model files directly in
      `Collars/`, else `file_type_disallowed`, strict-dependent); `tc` `deep/collar.rs`
      (`collar_findings`: `collar_<ID>` letter case aside, parsed to `u8`; 105 and 77 first,
      `collar_id_conflict` naming `FPC` or `referees`; then `1..=last_stock_collar(version)`,
      else `collar_id_invalid`; then `file_findings` as any model, an Error dropping the
      file). `fpc` became a `tc` dependency (`kit_values().collar`). TC-CMN-02, TC-CMN-10,
      TC-REF-07's collar half (`check`); TC-CMN-08's PES 17 refusal tested, uncited (its
      THEN needs the compile). For 4.9b2: with the strict check off a non-model file stays
      in `export.collars` (Info), so planning must pass over it. `mutants-diff 1d1ae5c`: 23,
      19 caught, 3 unviable, 1 missed (`named_id`'s prefix test), closed by the lead's
      `player_12` case.
    - [x] 4.9b2 the Fox compile: done 2026-10-07 (Opus 5.5, first time, no lead fix).
      `plan/collars.rs` `export_collar` (the run-wide `claimed_collars` map in `plan_run`,
      FMDLs in path order, `collar_id_conflict` naming the claimant's display name, the
      export itself for its second collar); `TaskKind::Collar { file, id }` after the logo,
      bytes unchanged at `paths::collar`; `TeamKitEdits { fpc, collar }` carried by
      `TaskKind::Kit` and `TeamKits` (an eighth parameter would have tripped
      `too_many_arguments`); the kit task sets the collar after FPC, before
      `reserved_collar_field`; `kit_configs` `edit_absent_slot` (FPC, then `wear_collar`,
      both through `edit_config`); `subset::collar_file` (`Compiled` FMDL, `NotCompiled`
      `.model`/glTF, `PassedOver` the rest), shared by the gate and planning; a refs
      export's collar named after its portraits. TC-CMN-01, TC-CMN-03. Only the kits the
      team's `UniColor.bin` record lists get the collar (the bundled base also holds
      `714_DEF_8th`/`9th`, which keep theirs). Left open, no real export reaches them: a
      claimant whose collar task fails still has its kits name the ID (the stock model
      shows); `collar_id_conflict` names the claimant by display name (an archive's stem),
      `duplicate_aesthetics_export` by file name. `collar_file` ignores the engine: safe
      while the gate refuses a pre-Fox target whole, to change with 4.16. Gates green (208
      of 254); `mutants-diff baa2391`: 44, 32 caught, 12 unviable, 0 missed.
    - Moved out: compiling a collar for pre-Fox (TC-CMN-08's compile half) with 4.16; a
      collar in the other engine's format (TC-CMN-09: an FMDL compiled for PES 17) with
      4.17's conversion; a glTF collar with Phase 7's glTF reading.

- [x] 4.10 **Kit layout conversion**: `KIT_LAYOUT_REMAP` (the plan's four sock bands; the
  shorts are not re-laid) in `processing/kit_layout.rs`, `kit_layout_converted`, each band
  resampled along u from its own rectangle (Lanczos3), every mip level the source carries, the
  blocks no destination rectangle touches kept, `_chest`/`_back`/`_name`/`_leg` untouched,
  placeholder never re-laid, the Fox → pre-Fox direction tested at the function (its CLI test,
  TC-KIT-19, comes with pre-Fox targets at 4.16). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Kits, "Layout conversion"); `player_folders.md` "Kit layout marker". IDs:
  TC-KIT-18, TC-KIT-20. Crates: tc → verify: a `pre-fox` kit compiled for PES 21 decodes to a
  texture whose every texel outside the sock rectangles equals the no-marker compile's and
  whose sock stripes' centres sit within 6 px of the golden; a synthetic flat-band texture
  round-trips pre-Fox → Fox → pre-Fox exactly
  - lead first, done 2026-10-04: the measurement (`scripts/provenance/kit_uv/`:
    `kit_uv_sock_angle.py`, `kit_uv_pants_height.py`, `kit_leg_atlas.py`), the plan's numbers
    and the golden (`tests/fixtures/kit_layout/`, written by `kit_layout_fixture.py`).
    Finding: the shorts are laid out alike in both engines; the earlier 36 to 60 px "shift"
    was the Fox body's height (decision entry)
  - done 2026-10-04: `dds_convert::resize` (`resample.rs`; Lanczos3 through `image`, the
    suite's one resampler); `processing/kit_layout.rs` (`KIT_LAYOUT_REMAP`, `relaid`); the
    kit task re-lays a marked kit's main texture and converts it outside the converter's
    cache, `kit_layout_converted` (from, to); the gate no longer refuses a `pre-fox` kit;
    help paragraph; `tests/cli/kit_layout.rs`. TC-KIT-18 and TC-KIT-20 proven (stripe
    centres within 4.5 px of the golden, 6 allowed). Not covered: a marked kit whose main
    texture is inherited from `all/`, raster and BC3 sources from the CLI, the lower mip
    levels in TC-KIT-18 (blocks straddling a rectangle's edge are encoded afresh there), a
    BC7 source keeps no blocks (encoded whole, as an unmarked raster is), and the mask and
    srm, which are re-laid when their steps emit them (4.16 for the mask). Gates green (123
    of 209); `mutants-diff 482c92d`: 150, 132 caught, 14 unviable, 4 missed (a block touching
    a rectangle's edge counted as reached), closed by a test of `reached_blocks`

- [x] 4.11 **Team root artifacts and Common**: logo (main decoded, made square per tag, Lanczos3 to
  512 and 256, `logo_small*` or main to 128, PNG with alpha, `logo_fit_applied`, `logo_upscaled`,
  one atomic producer, names `emblem_0{id}_r_ll/_r_l/_r.png` on PES 15-19 and `e_000{id}...` on
  20-21, `_r_ll` 512, `_r_l` 256, `_r` 128); `notes.txt` → `output/teamnotes.txt` (UTF-8/LF,
  canonical order, accepted exports only, `notes_found`); `kitN` variants completed
  (`kit_variant_missing` copies
  the lowest, `kit_variant_model_fox`), `dummy_kit*` stems skipped by the existence checks;
  `Common/` on Fox: textures converted into `common/{team_id}/sourceimages/`, models only
  reachable through `.common` links (4.5; 4.5c already emits `Common/`'s `.dds`/`.ftex` as one
  task); texture `.common` links (`hair.dds.common`: the player's stem resolves into Common and
  its path names the team's Common output; found at 4.5c with no step and no acceptance ID, so
  this step writes the scenario first). Plan: `pipeline.md` "4. Per-export non-model steps"
  (Logo, Common, Kit-dependent assets), "2. Per-export serial steps" step 5; `model_format.md`
  "Kit-dependent assets". IDs: TC-ROOT-06..09, TC-CMN-04..06. Crates: tc
  (`processing/team_assets.rs`) → verify: a 1000x600 `logo.png` yields three square PNGs decoding
  to 512, 256 and 128 pixels with transparent side borders and `logo_fit_applied` (`fit`);
  `logo_small_crop.png` beside it changes only the 128 one; the notes of two accepted exports
  appear in `output/teamnotes.txt` in canonical order and a skipped export's do not.
  Worked as four slices, each its own commit (decision entries of 2026-10-04: the logo's
  geometry; `teamnotes.txt`; kit variant sets):
  - [x] 4.11a the logo (TC-ROOT-06..08): done 2026-10-04 (Opus 5.5, first time, no lead
    fix). `dds_convert::encode_png` (`raster.rs`); `TaskKind::Logo`, one task per export
    after its kits, scoped on the main file, `DropFile` when it fails;
    `processing/team_assets.rs` (`logo`, `squared`, `centre_square`, `letterboxed`);
    `paths::logo`; `logo_fit_applied` (I) and `logo_upscaled` (W); the gate no longer names
    a logo; help. Red runs and five perturbations in the sidekick's report (padding before
    resampling shows as `[5, 0, 0, 5]` where `[0, 0, 0, 0]` is expected at (0, 100) of 512).
    Not covered from the CLI: sources other than PNG (a DDS at the function), the `emblem_0`
    names (PES 18 and 19; at the function), a tall `stretch` source. Gates green (126 of
    209); `mutants-diff 0e3739b`: 103, 94 caught, 7 unviable, 2 missed, both equivalent
    (`width > height` against `>=` in `centre_square` and `letterboxed`, which are called
    for a non-square image only)
  - [x] 4.11b the notes (TC-ROOT-09): done 2026-10-04 (Opus 5.5, first time, no lead code
    fix). `CheckedSource::notes` (the text read while the source is open, BOM removed);
    `plan_run` takes `ExportToPlan` tuples and fills `BuildManifest::notes` for the exports
    it keeps; `output/teamnotes.rs` (`render`, `write` through `.teamnotes-<pid>.tmp`);
    `compile::run` writes or removes the file once the CPK is promoted;
    `teamnotes_write_failed` (E, on the run); the test sandbox's teams list gains `/a/` and
    `/b/`; help. One brief premise was wrong (a `/b/` skipped by
    `players_txt_slot_duplicate` is dropped by validation, so no collection point could
    show its note; the planning test uses an export the gate skips instead). Known shape:
    `ExportToPlan` is a four-field tuple alias, kept because 27 test call sites build it
    inline; a struct is the next step if it grows. Not covered from the CLI: the removal
    failing, the temporary file failing to be written, a `--no-deploy` run with notes. Gates
    green (127 of 209); `mutants-diff f0f9002`: 19, 16 caught, 3 unviable, 0 missed
  - [x] 4.11c kit variants on Fox (TC-CMN-04, TC-CMN-05): done 2026-10-04 (Opus 5.5, first
    time; one lead fix, a test comment). `kit_variants.rs` (`kit_token`, `variant_stem`,
    `kit_number`, `model_variant_sets`); `TaskKind::Textures` and `CommonTextures` carry the
    export's kit numbers and `texture.rs` `complete_kit_variants` fills each set's gaps with
    the lowest variant's bytes (`kit_variant_missing`, W); `point_texture` points a `kitN`
    path at the part's textures when a variant of its set is among them;
    `PlayerFile::LeftOutKitVariant` for a higher per-kit model, which no task reads, and
    planning's `kit_variant_model_fox` (W) once per set and folder; help. TC-CMN-05 was split:
    it is the PES 21 half, and the PES 17 half is the new TC-CMN-07 (4.14), so no scenario
    counts as proven by half. Left as they are, each rare: a per-kit `.common` model link is
    not left out (the links merge like any parts); a skeleton named after a left-out variant
    has no model to pair with, so the gate names it; `check` does not print
    `kit_variant_model_fox` (a planning finding, as `link_combined` is); a reference is
    matched as spelled (`Pants_kitN` beside `pants_kit1.dds` is not). Not covered from the
    CLI: per-kit models in a reserved subfolder or a shared folder, a Common set completed
    (at the function), a set across a player's folder and a combined shared folder.
    Gates green (129 of 211); `mutants-diff 2249bf1`: 61, 48 caught, 13 unviable, 0
    missed. TC-CMN-06 (`dummy_kit*`) is
    4.29's, which builds the checks it is about
  - [x] 4.11d texture `.common` links (TC-TEX-09): done 2026-10-04 (Opus 5.5, first time, no
    lead code fix). `PlayerFile::CommonTexture(stem)` for a link to a texture, anywhere a
    link resolves (not in the `common/` subfolder, where validation checks no link);
    `processing/model.rs` `point_texture` takes the places a part's textures may be, in
    order: the folder's own stems at its texture home, then the linked stems at the team's
    Common output, a kit reference resolving in each; the gate names only material links
    now; help. Two contradictions: the `common/` exclusion (applied by the sidekick,
    accepted), and the brief's premise that validation refuses every stem held two ways
    (wrong for a combined shared folder: "Issues"). Not covered from the CLI: per-kit
    textures linked one by one (at the function). Gates green (130 of 211);
    `mutants-diff 2b0a388`: 26, 22 caught, 4 unviable, 0 missed

- [x] 4.12 **`ingame_face` processing (Fox)**: no face package emitted; arbitrary-named models
  rerouted to the player-exclusive boots folder (merged, the paired `.skl` becoming `boots.skl`);
  gloves parts to the player's gloves folders; a boots/gloves link combined with local parts
  under the marker; an empty `face/` ignored; a player without face models and without the marker
  gets the blank face folder (its contents per engine: `pipeline.md` step 4), always with the
  bundled face diff; a face file left in a folder with no face model is `face_file_not_used`
  (Info; 467 face folders of the VGL26 exports hold one, `.tmp/4_7/face_diff_blank_census.py`).
  Plan: `player_folders.md` "`ingame_face` marker", "`ingame_face` with shared links";
  `pipeline.md` "2. Per-export serial steps" step 4. IDs: TC-MOD-16..19, TC-MOD-32 (new).
  Done 2026-10-04 (sidekick, one brief, landed with two lead fixes to tests): the reroute is
  one branch of `plan/subset.rs` `model_role`, the marker carried by `FolderModels`
  (`of_player_files`) and `ModelFolder::ingame_face`, so the gate, `link_combines`, the ids,
  the findings and the tasks agree; `PlayerFile::UnusedFaceFile`, reported by
  `validation.rs` (`check` too); `plan/mod.rs` `folder_tasks` plans the face of every
  unmarked player folder (`blank_face`), and `processing/model.rs` `package` builds the
  blank one unchanged; the gate no longer names the marker or a model-less folder; help;
  `tests/cli/face_folders.rs`. The pre-Fox blank folder comes with 4.14. TC-MOD-09 is
  reworded (decision entry): a failed package is left out alone, the blank face and the
  textures still packed ("Phase 4 open questions"). Known, not fixed: under the marker a
  texture of the player's own that differs from a combined `Boots/` folder's is
  `shared_texture_conflict` (dropped=boots) where both feed the boots, by the rule that a
  player's own textures stand for its face (the folder's models are left out either way); a
  marked folder holding only textures emits nothing and reports nothing; a `Faces/` folder
  with no face model also gets `face_file_not_used` from `check` (`compile` refuses the
  folder). Not covered from the CLI: those three, and a marked folder whose rerouted models
  bring different skeletons (`skl_merge_conflict`, seen by the sidekick's probe). Gates
  green (135 of 212); `mutants-diff 03d4fae`: 50, 43 caught, 7 unviable, 0 missed

- [x] 4.13 **Run planning**: `duplicate_aesthetics_export`; canonical export order by normalized
  source-relative path plus source kind; every task's output namespace allocated in planning,
  `duplicate_path` (folder E, export E, override W); the `overrides/` tree of the data directory
  injected first (`overrides_active`); the writer's duplicate invariant kept; export revision
  pinning at planning, `source_changed_during_run` (a folder's revision is the listing's (path,
  size, modified time) set, an archive's its size and modified time, pinned at planning and
  rechecked before each task's read). Plan: `pipeline.md` "3. Per-model-folder parallel steps"
  (the run-level planning paragraph), "5. Writer" steps 1-3, "Resolved decisions" (Cross-export
  duplicate precedence, Source snapshot); `settings.md` "Path resolution" (`overrides/`). IDs:
  TC-PLN-03..07. Crates: tc (`plan/mod.rs`, `plan/overrides.rs`, `reader/`) → verify: exports
  `co - A/` and `co - B.zip` in one root both report `duplicate_aesthetics_export` and no CPK is
  written; with `overrides/common/etc/TeamColor.bin` in the data directory the CPK's entry has
  the override's bytes and the console shows `overrides_active` then `Warning duplicate_path
  [Keep]`. Worked as three slices (decision entry of 2026-10-04):
  - [x] 4.13a duplicate exports and the export order (TC-PLN-03, 05, 07), done 2026-10-04
    (sidekick, landed first time): `validation.rs` `refuse_duplicate_teams`, after every
    source is checked, so `check` reports it too: each export of a team several resolve to
    gets `duplicate_aesthetics_export` (context `id`, `exports`) and is not planned; the
    export order is discovery's, unchanged. Two exports of one team with disjoint content
    used to compile into one CPK with no finding, and colliding ones aborted the run at the
    writer. TC-OUT-03's test now blocks the staging folder with a file to get its failed
    CPK write (it used two exports of one team); no CLI test reaches the writer's duplicate
    invariant any more (its unit test stays). Not covered: an export dropped by an exhausted
    ID pool beside another of its team (no duplicate finding, by the rule that only a
    resolved export counts); a folder beside a `.7z` through `compile`. The sidekick looked
    for two tasks emitting one CPK path and found none (paths are keyed by team, player,
    boots or gloves id, kit name or folder name, and the in-export cases are refused or
    merged before tasks). Gates green (138 of 212); `mutants-diff d053ac9`: 7, 6 caught, 1 unviable, 0 missed
  - [x] 4.13b the `overrides/` tree and `duplicate_path` (TC-PLN-04), done 2026-10-04
    (sidekick, landed first time; one lead fix, a help line rewrapped): `plan/overrides.rs`
    `list` (every file below `<data dir>/overrides/` by CPK path, and the `overrides_active`
    note), listed by `compile` before any export is read; `output/writer.rs` `CpkOutput`
    adds them when it creates the CPK, in path order, each read as it is added, and creates
    the CPK at `finish` when nothing else was committed; `add` leaves out an entry or a bin
    at an override's path and reports `duplicate_path` (Warning, on the run) among that
    task's messages or `finish`'s. A run with overrides and no export compiled now writes
    and promotes a CPK holding the overrides and the bins. Known, not fixed: a bin an
    override replaces is still built, so `bin_header_repaired` is still reported for it and
    a failed kit config insert still fails the run (neither reachable on the bundled bins;
    to settle with 4.21's installed bins). Paths compare exactly, by the maintainer's
    ruling (decision entry of 2026-10-04: no case folding). Not covered from the CLI: an `overrides/` tree that cannot be listed or a
    name that is not UTF-8 (exit 3), an unreadable override (unit-tested at the writer), an
    override at `UniformParameter.bin` or `UniColor.bin`. Gates green (139 of 212);
    `mutants-diff 62d8e66`: 28, 16 caught, 12 unviable, 0 missed
  - [x] 4.13c source pinning, `source_changed_during_run` (TC-PLN-06), done 2026-10-04
    (sidekick, landed first time, no lead fix): `reader/source.rs` `list` also returns the
    source's `SourceRevision` (a folder: each file's size and modified time, from the
    metadata the walk already asks for; an archive: the archive file's), carried by
    `Route::Validate` and `CheckedSource`; `compile.rs` `coordinate` asks
    `SourceRevision::changed` after each task's read (a `.7z` once, after its one read) and
    stops at the first change; `run` is `plan` then `build`, and `build` discards the
    staging and reports the Fatal on the export, naming the file. A file gone before its
    read is a change, not a `source_read_failed`. TC-PLN-06's test replaces `boots.fmdl`
    (nothing staged yet) and, in a second pass, a kit texture (a staged CPK exists and must
    go): with the boots alone the staging assertion could not fail. Not decided: an aborted
    run sends no `ExportProcessed` for the exports it did not finish (as after
    `cpk_write_failed`; the console prints nothing for it, Phase 8's grid may want it);
    events still carry no export revision (Phase 8). Not covered from the CLI: a `.7z` or
    `.zip` changed during a whole compile (covered at `coordinate`), a change in a second
    export after the first one's batches were committed. Gates green (140 of 212);
    `mutants-diff 527423c`: 26, 15 caught, 11 unviable, 0 missed

- [x] 4.14 **Pre-Fox faces (PES 15-17, native `.model` + `.mtl`)**: `face.xml` generated from the
  suffix table (`face_neck` for `face_high`, `parts`, `gloveL`/`gloveR`, `handL`/`handR`,
  `model_type_<x>`, `_ratio_<n>`, `uniform` → `uniform_sub` on PES 15), `oral_`/`_win32` affixes,
  `xml_face_neck_added` with the dummy model and MTL (lead first: both as lead-authored fixtures
  with a provenance README); `.mtl` texture paths rewritten to
  `model/character/uniform/common/{team_id}/{folder}/` with `.dds` extensions; textures as DDS
  (BC7 transcoded to BC3, `texture_not_div4`); each face packed into
  `common/character0/model/character/face/real/{id}.cpk`; portraits `player_{id}{NN}.dds`; a
  shared face folder copied per linking player with local files on top; boots/gloves links keep
  the shared folder's ID, the shared folders emitted as loose files under `boots/{id}/`; local
  boots/gloves parts ride in the face XML (no per-player folders); `model_name_invalid`,
  `edithair_unsupported`, `xml_oral_prefix_missing` (PES 16), `.model.common` as an XML Common
  path, `xml_common_path_invalid`. Plan: `pipeline.md` step 3 "Pre-Fox fixups", steps 5-7, "Game
  paths reference"; `player_folders.md` "A link plus local models combines" (pre-Fox), "Model
  names". (The pre-Fox parity reference for 4.14-4.17 is step 4.31's, the lead's; until it
  exists the step's checks are the ones below.)
  Also the pre-Fox half of the kit variants (4.11c did Fox): a model variant set as one
  `face.xml` entry naming `…kitN…` with the variant files beside it, and the texture sets
  completed as on Fox.
  Also the pre-Fox blank face folder and the pre-Fox half of `ingame_face` (4.12 did Fox):
  `pipeline.md` "2. Per-export serial steps" step 4 gives the blank folder's contents, and
  `face_file_not_used` is Fox-only until then.
  Settled 2026-10-07 (the two former "Open first" items): which `.mtl` a `.model` uses,
  restated from Red's `find_mtl_file` (`model_format.md` "Pre-Fox: the `.mtl` a `.model`
  uses"; TC-MOD-24 pins the override and the Common arm); a model's type read from its stem
  without its kit token (`pipeline.md` "Kit-dependent assets"), which changes Fox too
  (`boots_kit1.fmdl` is boots, not face content): its first slice.
  Slices, each its own commit: (a) the kit-token typing, both engines; (b) a player folder's
  own `.model`/`.mtl` files compiled for PES 15-17 (the pre-Fox refusal lifted for them):
  roles, the `.mtl` search over the file's and the model folder, the generated `face.xml`
  (type table, `oral_`/`_win32` and the `*` path, `ratio`, `model_type_<x>`, the dummy and
  `xml_face_neck_added`, `<dif>`), `.mtl` texture paths and DDS textures in the player's
  common subfolder, the nested face CPK, local boots/gloves riding in the face XML, the
  pre-Fox blank face folder (TC-MOD-20, 21, 39); (c1) shared folders: a face link combined
  into the face, local files on top, shared boots/gloves folders as loose files
  (`boots.model`/`boots.mtl`, gloves under their own names with `glove.xml`), no item rows
  on pre-Fox (TC-MOD-22, 38, 40); (c2) Common: `.model.common`/`.mtl.common` and texture
  links, the pre-Fox Common output (TC-MOD-24, 37); (d) the name and limit
  checks: `model_name_invalid`, `edithair_unsupported`, `uniform_sub` with
  `xml_uniform_pes15`, `texture_not_div4`, the vertex limit, a material a `.model` names
  missing from its `.mtl` (`check_bundle`'s rule, in the deep pass)
  (TC-MOD-23, 25, TC-CHK-08, TC-TEX-07, TC-XML-08; `xml_oral_prefix_missing` is a user
  `face.xml` check, 4.15's, with TC-XML-09); (e1) `ingame_face`'s boots through
  `pes_model::ops::merge` (the player's boots parts and a combined boots link merged into a
  player-exclusive `boots.model`, `model_merged`, the merge conflicts with
  `model_merge_flags_conflict`) and a shared boots folder holding several boots models merged
  into its `boots.model`; an `ingame_face` folder holding gloves parts stays refused
  (TC-MOD-35, 41); (e2) `ingame_face`'s gloves (unmerged, `player_folders.md` "`ingame_face`
  with shared links"; `ModelFolder::own_package` then splits the player's `.mtl` files and
  textures between his boots and gloves; the boots task's two package guards in
  `prefox_shared::package`, e1's mutation survivors, get their test here: a marked player with
  boots parts and a gloves link his gloves combine); (e3) the hand split of a hand-weighted
  `.model` in the face task (`model_conversion/hand_split.md` "Pipeline integration": the
  body keeps its entry, the hands become `<stem>_glove_l`/`_glove_r` entries; the deep pass
  records `.model` hand weights through a `pub` `.model` detector in `model_convert` beside
  `fox_has_hand_weights`, a two-crate change: GPT review (b) queued; lead first: the
  `.model` twin of `tests/fixtures/hand_split/body.fmdl`; TC-MOD-43); (e4) the kit variant
  sets (TC-CMN-07); (e5) a `.model.common`/`.mtl.common` link
  beside `ingame_face` (named by the gate since e1: its model would be one more part of the
  boots merge, or a `glove.xml` entry naming its Common path). A user-supplied `face.xml` stays refused as not compiled yet until 4.15.
  Templates for (b): `resources/templates/dummy.model` and `dummy.mtl` (README there).
  (a) done 2026-10-07 (Opus 5.5, first time, no lead fix): the token grammar moved to
  `aesthetics_export::conventions::kit_token` (`KitToken`, `kit_token`, `variant_stem`,
  `without_kit_token`), `model_suffix` reads the stem without the token; a shared
  `Boots/` folder's `boots_kit1`/`boots_kit2` now pass `check` and compile kit 1's alone
  (`kit_variant_model_fox`). Gates green (209 of 254); `mutants-diff 7b13269`: 44, 39 caught,
  5 unviable, 0 missed; Clef: two flags, both rejected.
  (b) done 2026-10-07 (Opus 5.5, one rework round of five findings, no lead fix):
  `face_xml.rs` (type table, packed names, `ratio`, the XML bytes), `mtl_search.rs`
  (`mtl_for`, shared by the deep pass and the face task), `processing/prefox_face.rs`;
  `PlayerFile::PreFoxModel`/`Material`, `FolderModels` carrying the engine, the pre-Fox
  gate `pre_fox_not_compiled`; `TextureHome` per engine; the nested CPK in `materialize`
  (`EntryTarget::GamePaths { engine }`); `dummy.model`/`dummy.mtl` templates;
  `xml_face_neck_added`; `model_material_undefined` (no `.mtl` found, pre-Fox only);
  `aesthetics_export::ends_with_name` (the suffix walk, shared with the native types).
  TC-MOD-20, 21, 39. Gates green (212 of 254); `mutants-diff bc3cd33`: first pass
  153, 2 missed (rework 5); rerun 153, 0 missed (125 caught, 25 unviable, 3 whose remote
  builds the 9 GiB cap killed, rerun locally: caught; `REMOTE_BUILD_JOBS` 2 → 1). Clef: 0 flags.
  (c1) done 2026-10-07 (Opus 5.5, first time, no lead fix; one contradiction accepted: the
  gloves `.mtl` names lowercased, as the plan says): a face link feeds the player's face on
  pre-Fox (`link_feeds_own_package`), the gate walks a linked `Faces/` folder and each shared
  boots/gloves folder taking an id (`pre_fox_shared_not_compiled`: one boots model, model,
  `.mtl` and texture roles only); `prefox_face` searches each source's own files and leaves
  out a shared file whose packed name the player's packs; `processing/prefox_shared.rs`
  (`boots.model`/`boots.mtl`, gloves with `face_xml::glove_xml`); `paths::pre_fox_package_folder`
  and the pre-Fox `SharedOutput` home `./`; loose boots/gloves entries in `materialize`; no
  item rows on pre-Fox. TC-MOD-22, 38, 40. Gates green (215 of 254); `mutants-diff 0557132`:
  52, 49 caught, 3 unviable, 0 missed (remote peak 8.99 GiB, no build killed). Clef: one
  flag, rejected.
  (c2) done 2026-10-07 (Opus 5.5, one rework round of two findings, both from its own report;
  two contradictions accepted: the link roles' `package()` is `None`, and an unresolved
  shared-folder `.mtl.common` is no candidate): roles `PreFoxCommonModel`, `CommonMaterial`
  and `CommonTexture` on pre-Fox; `mtl_for` to the plan's full search (`.mtl.common` links,
  a model link's Common-first order); the deep pass searches only the `Common/` files its
  Common pass kept (a broken Common `.mtl` behind a link was a face-task panic); planning's
  `ModelFolder.common_files` and `TaskKind::CommonModels` (`processing/prefox_common.rs`);
  `paths::common_texture`/`common_texture_directory` per engine, `pre_fox_common_file`; the
  face's entries name Common models and `.mtl` files by their Common paths, one entry per
  linked model; `rewritten_materials` over several places. TC-MOD-24, 37. Gates green (217
  of 254); `mutants-diff f4f3a56`: 101, 86 caught, 15 unviable, 0 missed (the local half
  rerun alone after a PC crash killed it; the remote half's 50 collected). Clef: one flag,
  rejected.
  (d) done 2026-10-07 (Opus 5.5, first time; three contradictions accepted: no legacy tool or
  phase number in a comment, a finding's `file` below its folder; lead fix: one doc line):
  `model_name_invalid` (`check_shared`'s naming rule on both engines, one code each),
  `edithair_unsupported` (`edithair_files`, pre-Fox, any depth, player and shared folders);
  `face_xml::version_type` and `xml_uniform_pes15`; `texture_not_div4` in the pre-Fox size
  rule, never eligible; `model_material_undefined` for a named material (`ModelRead.materials`,
  the private `ContentPass::materials`, `KeptCommon`, `material_finding`: the reads stay one
  file per worker, the names compared after). TC-MOD-23, 25, TC-CHK-08, TC-TEX-07, TC-XML-08.
  Gates green (222 of 255); `mutants-diff 993a190`: 35, 31 caught, 4 unviable, 0 missed (two
  remote builds killed at the 9 GiB cap, rerun locally: caught).
  (e1) done 2026-10-08 (Opus 5.5, first time; one contradiction accepted: a merge's
  `MergeError::Other` is `folder_pack_failed`, no task loading a `.model` otherwise; lead fix:
  the merge charged to the memory budget as Fox's is): `PlayerFile::PreFoxPart { package }`
  (under the marker a model goes where Fox's `model_role` puts it; `own_package` is the boots
  for a marked pre-Fox player); `link_combines`/`holds_model` per engine, a pre-Fox boots link
  combining only beside a marked player's boots part; the gate's `compiled_under_ingame_face`
  (gloves parts and `.model.common`/`.mtl.common` links named); `prefox_shared::package` for
  both a shared folder and a player's exclusive boots, `boots_files` (one part as it is,
  several through `pes_model::ops::merge`), `From<MergeError>`; `prefox_face::read_materials`,
  `linked_texture_stem`; `model_merged`, `model_merge_flags_conflict`; `face_file_not_used`
  under the marker on pre-Fox. Tests in `tests/cli/prefox_ingame_face.rs`. TC-MOD-35, 41.
  Gates green (224 of 255); `mutants-diff 55ff16d` (local only, `LOCAL_ONLY_CRATES`): 78, 65
  caught, 11 unviable, 2 missed (the two package guards of `prefox_shared::package`, no input
  reaching them until gloves parts do: their test is e2's). The first run died with the PC
  (a BSOD); rerun whole.
  (e2) done 2026-10-08 (Opus 5.5, first time; one contradiction accepted, applied by the
  sidekick: planning routes each pre-Fox model and `.mtl` by its source, not by
  `own_package`, since a combined gloves folder's models went to the boots task; lead: a plan
  edit and decision for the gloves `.mtl` rule it changed): `PreFoxPart { package, xml_type }`,
  `compiled_under_ingame_face` accepting gloves; `folder_files` giving each file's source,
  `TaskKind::files`/`folder_tasks` routing by it (a marked player's `.mtl` files are files of
  both his tasks); `prefox_shared::package` writing a player's exclusive gloves as a shared
  folder's, `without_replaced` (his file over a combined folder's of the packed name), only
  the `.mtl` files the models use; `help/`. Tests in `tests/cli/prefox_ingame_face.rs`.
  Gates green (224 of 255); `mutants-diff 2addd6f` (local): 41, 36 caught, 5 unviable, 0
  missed, e1's two guards among the caught. Two open issues logged (texture stem conflicts
  under the marker, a replaced `.mtl` behind a kept linked model). The first run died with
  the PC (the third BSOD); rerun whole.
  (e3) done 2026-10-08 (Opus 5.5, first time; two contradictions accepted, both applied by
  the sidekick: a pre-Fox model named as boots or gloves is never split, which the brief's
  role rule would have (`subset::named_as_face`, `model_role` without the marker); the split
  reads its `.mtl` in place from the task's files, since a linked face's `.mtl` the player's
  shadows is never in the face's own `.mtl` list; one plan gap decided after its report: a
  split model whose `.mtl` is a Common file is refused, decision entry): `model_convert`
  `ops::hand_split::prefox_has_hand_weights` `pub` (moved from `convert.rs`); the deep pass's
  `.model` arm records hand weights; `hand_split_parts` by engine (pre-Fox: the face's own
  and a combined shared face's `.model` files named as face content, never a Common-linked
  one nor an `ingame_face` part), no gloves task on pre-Fox; `processing/prefox_split.rs`
  (`split_face_model`: `.model` + `.mtl` to the IR, `split_by_skeleton_group`, each part
  back to a `.model`, charged at the source's size; `model_hand_split`; a failure
  `model_conversion_failed`); `prefox_face::face` packs the body in its place and the hands
  as `oral_<stem>_glove_l_win32.model`/`_glove_r_` entries typed `gloveL`/`gloveR` with the
  body's `.mtl` and `ratio`; `help/`. Tests: unit (detector, deep, plan, split) and
  `tests/cli/prefox_hand_split.rs` (TC-MOD-43 and the Common `.mtl` refusal). A side effect
  worth knowing: a hand-weighted `.model` with `model_material_undefined` under
  `pass_through` now fails `model_conversion_failed` (the import needs every material) where
  it was packed as it was. Gates green (225 of 256); `mutants-diff b75d16a` (local): 29
  (`model_convert` 6, `team_compiler` 23), 24 caught, 5 unviable, 0 missed.
  (e4) done 2026-10-08 (Opus 5.5, one rework round of two findings, both raised by its own
  report: a set was found per source folder, so one split between a linked shared face and
  the player's own files was two sets of one, kit 2's model never listed, or listed twice;
  and the `.mtl` check compared names without directories; two contradictions applied by
  itself and accepted: `packed_model_name` lower-cases the reference, so the entry is
  respelled after packing; the strict lookup of a set's listed material panicked; lead fix:
  one unit test from the mutation survivor): a per-kit `.model` set is packed whole and
  listed once in the generated `face.xml` through its lowest variant's entries (its own, and
  its hands' when split), the kit token spelled `kitN` in `path` and `material`
  (`prefox_face::kit_places` over the face's packed names after the shared face's copy-in,
  `KitPlace`, `listed_material`, `respelled_material`); `kit_variant_mtl_differs` (W) for a
  variant whose `.mtl`, as the entry would write it, is not the listed one's respelled for its
  number; `point_materials` points a `kitN` texture reference at the place holding a variant
  (`kit_variants::has_variant_among`, shared with the Fox pointing); `face_xml::xml_type` reads
  the stem without its kit token (`aesthetics_export::without_kit_token` now `pub`); a per-kit
  `.model` has a pre-Fox role, and the gate names one under `ingame_face`, in a shared
  `Boots/`/`Gloves/` folder taking an id, or behind a `.common` link (decision entry). Open
  for 4.16: completing a model set against the kit numbers (its line). Tests:
  `tests/cli/prefox_kit_variants.rs` (TC-CMN-07, the `.mtl` off the rule, one `.mtl` for the
  set, a split set both ways, the listed variant sorting after an unlisted one, a Common
  `.mtl` against a face one), a hand-split set in `prefox_hand_split.rs`, units in
  `kit_variants`, `subset`, `face_xml`, `prefox_face`. Gates green (226 of 256);
  `mutants-diff 5964639` (local): 47, 40 caught, 6 unviable, 1 missed (`listed_material`'s
  guard; its unit test added by the lead, caught by hand with the guard perturbed).
  (e5) done 2026-10-08 (Opus 5.5, fresh agent, one rework round of one finding it raised
  itself as a plan gap: the Common-texture place also pointed his own `.mtl` files at an
  unlinked Common stem, which the plan resolves only through a link; one contradiction
  accepted without code: the deep pass reads a folder's roles without the marker, so its
  `.mtl` check already covered a marked folder's link; lead fix: none): under `ingame_face` a
  `.model.common` link named as boots or gloves is a `PreFoxPart` (`subset::pre_fox_part`,
  shared with the `.model` arm), planning puts the Common model in the link's place with the
  `.mtl` its search finds (`CommonModel::material`, pre-Fox only; `ModelFolder::roles`,
  `push_common`) and a `.mtl.common` link's Common `.mtl` beside it, so the boots merge and
  the gloves output read them as his files (`prefox_shared::material_of`); a copied Common
  `.mtl` alone names the textures directly in `Common/` at the team's Common output
  (`places_for`). Left as they are: his own file beside a Common one of its packed name fails
  the task naming the file; a Common `.mtl` is read by each task using it and by the Common
  models task. Tests: TC-MOD-44, TC-MOD-45 and the own-`.mtl` sibling in
  `tests/cli/prefox_ingame_face.rs`; units in `subset`, `plan` (`roles`, task files and
  charges), `deep`. Gates green (228 of 258); `mutants-diff db84354` (local): 37, 30 caught,
  7 unviable, 0 missed.
  IDs: TC-MOD-20..25, TC-MOD-35, TC-MOD-37..41, TC-MOD-43..45, TC-CHK-08, TC-TEX-07, TC-XML-08, TC-CMN-07 (split from TC-CMN-05 at 4.11c). Crates: tc (`processing/model.rs`, `processing/material.rs`,
  `processing/texture.rs`, `paths.rs`) → verify: a `/co/` slot 05 folder with the smallest
  `pes_model` fixture pair as `face_high.model` + `face_high.mtl` and `skin.dds`, compiled for PES 17: the
  CPK holds `common/character0/model/character/face/real/71405.cpk` whose `face.xml` lists one
  `face_neck` entry, whose MTL names `skin.dds` under `.../common/714/05 - .../`, and no `Asset/`
  entry. Closed 2026-10-09 at converge: the criterion is TC-MOD-20's test
  (`tests/cli/prefox_faces.rs`
  `a_player_s_face_model_compiles_into_its_face_cpk_with_the_generated_face_xml`: the nested
  `71405.cpk`, one `face_neck` entry, the MTL's `model/character/uniform/common/714/05 - A/skin.dds`,
  no `Asset/` entry), re-run green.

- [x] 4.15 **Pre-Fox XML and MTL checks, user `face.xml`**: the Error/Warning/Info line of
  "User-supplied `face.xml`" with its resolution paragraph, `xml_ignored_fox`,
  `mtl_texture_not_found` (mesh-used materials) against `mtl_texture_unused_missing`. The
  `mtl_material_duplicate` and `mtl_state_*` rows are already reported by the deep pass from
  `pes_model::check::check_materials` (4.14d). Deferred, not gaps: the states checks on a
  converted model's `[prefox.states]` wait for glTF sources to compile (Phase 7; the deep pass
  does not read glTF); `xml_texture_path_missing` needs the texture suffix mapping table of the
  same phase (today a sampler without `path` is `mtl_broken`); a `face.xml` in a shared face
  folder stays refused by the pre-Fox gate (open question in the plan paragraph); a pre-Fox
  texture `.common` link satisfied by an installed CPK (the face task points a link by its
  target's name; `installed_common_textures` stays Fox-only); a Common-form texture path in a
  `.mtl` packed with its `XXX` segment as written although the check calls it supplied (whether
  Red respells it: 4.31's pre-Fox parity). Plan:
  `messages.md` "XML/MTL content checks", "User-supplied `face.xml`". IDs: TC-XML-01..07,
  TC-XML-09 (TC-XML-08 is 4.14d's). Slices: (a1) the xml read, checked and resolved
  (`user_face_xml.rs`: parse, the content checks, reference resolution, serialization), its
  findings in the deep pass with the listed models' `model_material_undefined` compared against
  the entry's `.mtl`, the `FaceXml` role and a folder with an xml having a face,
  `xml_ignored_fox` on Fox, the catalog rows (TC-XML-03, TC-XML-06, TC-XML-09, TC-XML-04's
  first half); (a2) the face task emitting from the xml (references packed as the plan says,
  the dummy and the version rewrite as for a generated xml, the `<dif>` sources), the gate
  lifted for a player folder's own xml, the Fumos xml of the pre-Fox tracer compiled verbatim
  as a second test (TC-XML-01, TC-XML-02, TC-XML-04, TC-XML-05); (b) `mtl_texture_not_found`
  and `mtl_texture_unused_missing` in the deep pass under Fox's rule ("A texture a model names
  must exist"), which pairs each model with its `.mtl` and has read both: a `./` or bare path
  whose stem the folder does not hold is missing, a Common-form path is looked for in `Common/`
  and the installed CPKs, the installed stems reaching the pass from validation (TC-XML-07; the
  paragraph after "Texture existence is checked **deep**"). Crates: tc → verify: the hand-written xml of `testing.md` ("user
  `face.xml`") compiled for PES 17 is emitted with 714 substituted into its Common path, its
  unknown `type` and extra attribute kept with `xml_type_unknown` and `xml_attribute_unknown`,
  `level="1"` kept with `xml_level_lod`; a `<model>` without `path` drops the folder with
  `xml_model_path_missing`
  (a1) done 2026-10-08 (Opus 5.5, fresh agent, landed first time; four departures from the
  brief, all accepted: `check` takes the xml's name for `xml_dif_conflict`'s context,
  `XmlError::Xml` carries the text's end position for the three parser errors roxmltree places
  nowhere, the Fox gate lets a face's own `face.xml` through (TC-XML-06 needs it), the pre-Fox
  gate names a shared face's own xml; lead fix: none): `user_face_xml.rs` (`parse` to
  `UserFaceXml { children: Child::{Model, Dif, Other} }` keeping `<model>` attributes and
  unknown elements verbatim, `reference` → `Local`/`Common`/`Unchecked`, `resolve` over the
  folder's own files then the linked face's, a `.mtl.common` link counting as a `.mtl`, a
  `kitN` name found through its lowest variant, `check` with every `xml_*` row but the two
  rewrites); `face_diff::from_dif` shared with `face_diff.xml`; `PlayerFile::FaceXml`, a folder
  with an xml having a face, the xml under `ingame_face` unused; the deep pass reads each own
  xml (`xml_broken`/`xml_root_tag_invalid`/`face_diff_invalid` at parse, `check`'s findings
  after) and compares only the listed models' materials with the entry's `.mtl`
  (`undefined_materials` split from the search); `xml_ignored_fox` from the structure pass; 16
  catalog codes (`ALL` 109); help. Gates 232 of 258 proven (TC-XML-03, 04, 06, 09).
  `mutants-diff d4ea252` (local): 184, 150 caught, 30 unviable, 4 missed (the linked face's
  files never reaching the deep pass, the two guards of `listed_materials`, `resolve`'s kind
  check on a Common reference), each covered by a lead test; rerun 154 caught, 0 missed.
  (a2) done 2026-10-08 (Opus 5.5, fresh agent, landed first time; one ruling it changed with
  evidence: the source loop's model arms call the `.mtl` search, which `expect`s a hit the deep
  pass no longer promises for an xml folder, so both arms are skipped there; three
  contradictions accepted without code: the task's `common_files` hold no `Common/` `.model`,
  so a Common reference is written unchecked on the deep pass's guarantee; a Common `.mtl` keeps
  the reference's spelling; `model_hand_split` is a processing finding, so the plan test asserts
  an empty `hand_split`; two gaps applied and accepted into the plan paragraph (a `kitN` `.mtl`
  set packs every variant; any file named twice packs once); one left (an xml naming the dummy's
  names without a `face_neck` fails the task); lead fix: none): `face_xml::user_face_xml` writing
  `WrittenChild::{Model, Other}` through the same line writer as a generated xml
  (`push_element`, `push_tail`; `XmlEntry::attributes`); `prefox_face::user_xml_face` with
  `XmlFace` (a `./` file packed under the referenced name, a `kitN` set whole under the folder's
  names, a Common model written `oral_<stem>_*.model` at the team's Common output, a Common
  `.mtl` by its name there, `.mtl` files pointed as a generated face's, the dummy appended with
  `xml_face_neck_added`, `xml_uniform_pes15` naming the entry's `path`, the xml's last `<dif>`
  else the folder's else the bundled one); `ModelFolder::own_face_xml` (no hand split for such
  a folder); the pre-Fox gate lets a player folder's own xml through. Gates green (TC-XML-01,
  02, 04, 05; the Fumos xml compiled verbatim as a test). `mutants-diff 13d0750` (local): 57,
  51 caught, 3 unviable, 3 missed (the serializer's empty-element guard with text alone or a
  child alone; the kind check of the `kitN` packing loop, which the CLI test hid by naming
  both sets), each covered by a lead test; rerun 54 caught, 0 missed.
  (b) done 2026-10-08 (Opus 5.5, fresh agent; one rework round, caused by the plan, not the
  code: the row's Error refused the pre-Fox tracer's Fumos face, a cup-played export whose
  `face.mtl` names `./face_edithair_specular_roughness.dds` on a mesh-used material it holds no
  file for, which Red warned on and compiled; the sidekick reported it as a blocker and
  loosened nothing, the lead ruled the finding a Warning (DECISIONS); one contradiction it
  applied on plan evidence, accepted: mesh-used means bound by a mesh, not listed by the model
  (`MaterialRead::mesh_used`); six gaps reported, two fixed in the rework (a referee's combined
  boots and gloves hold textures for his face; no texture finding on a folder an xml Error
  drops), one deferred (the `XXX` segment, above), three what the plan says; lead fixes: the
  parity test's expected warning and one test comment): the deep pass keeps each pre-Fox
  material's sampler paths (`deep::model::MaterialRead`) and looks every `.mtl`'s up by the
  face task's stem rule (`deep::materials`: `supply`, `texture_findings`, `held_stems`), each
  model paired once with its `.mtl` (`deep::Pairing`, `pairings`) for both
  `model_material_undefined` and the mesh-used set, `FaceUse` naming the shared folders whose
  textures a face packs, the installed Common stems reaching the pass from `check_source`
  (`InstalledPaths::common_texture_stems` by engine), a `Common/` `.mtl` checked once against
  the Common models' names; `tests/cli/prefox_textures.rs` (TC-XML-07, TC-TEX-05's three runs
  on PES 17, a game path and a dummy stem). Gates green (236 of 258). `deep/mod.rs` is past
  2,000 lines: converge's design-health pass should move the pairing and the texture wiring
  beside `deep/materials.rs`. `mutants-diff a98219b` (local): 72, 59 caught, 13 unviable (each a
  `Default::default()` on a type without one, or `||` inside a `let` chain), 0 missed.

- [x] 4.16 **Pre-Fox kits, bins and DDS compression**: a kit marked `fox` has its main texture
  and its own mask re-laid to the pre-Fox layout (`kit_layout::relaid`, TC-KIT-19), `kit_mask` injected from the mask template
  (lead first: the template as a lead-authored fixture with a provenance README) when absent,
  `kit_srm` dropped with `kit_texture_not_used`, kit configs emitted as loose per-team bins under
  `uniform/team/{team_id}/` (no `UniformParameter.bin` before PES 18), `UniColor`/`TeamColor`
  in the one layout every version shares (`resources/bins/README.md`), `fox` kits re-laid with the inverse table (4.10);
  `dds_compression` (`auto` follows `multicpk_mode`, `true`, `false`) wrapping every emitted DDS
  with `wezlib::compress` on PES 15-17 only, already-wrapped sources passed through, the level
  chosen by measuring levels 1, 3 and 6 on the tracer's DDS set and recorded in a decision entry.
  Open for the maintainer (`docs/QUESTIONS.md` "Per-kit model sets on pre-Fox"). From the
  tracer's parity (4.16a): Red's kit config keeps the member's names for the four number
  textures the export does not ship (`u0731g1_back`, `_chest`, `_leg`, `_name`), where the
  compiler's encoder leaves an absent texture's name empty, on Fox as on pre-Fox. Whether PES
  15-17 treat an empty name and a name with no file alike (the default numbers either way) is
  an in-game question; the parity row compares the config outside those four fields until it
  is answered.
  Plan: `pipeline.md` "4. Per-export non-model steps" (Kits: mask and srm); `settings.md`
  (`dds_compression`, "DDS compression cost"). IDs: TC-KIT-21..23, TC-KIT-28..29, TC-TEX-08,
  TC-BIN-04, TC-BIN-18. Crates:
  tc (`processing/kit.rs`, `processing/texture.rs`, `bins/`, `settings.rs`) → verify: PES 17
  compile of `/co/` p1 without a mask emits `u0714p1_mask.dds` byte-identical to the template;
  with `dds_compression = true` every `.dds` entry satisfies `wezlib::is_wrapped` and
  decompresses to the `false` run's bytes; a `kit_srm.dds` reports `kit_texture_not_used` and no
  `_srm` entry exists. Also, from 4.9b: a `.model` collar compiled for pre-Fox at
  `common/character0/model/character/uniform/nocloth/collar_012.model` (`paths::collar`
  and `subset::collar_file` taking the target's engine), the team's loose configs naming
  it; TC-CMN-08's compile half. Also, from 4.32: TC-KIT-26 whole (a row `kit_back`
  compiled for PES 17 becomes a column in digit order), cited once both halves run.
  Slices: (a) pre-Fox kits and bins: the gate lifted for kits, kit textures at the pre-Fox
  path, the mask emitted (the kit's own, inherited, or the bundled template as it is), the srm
  on Fox emitted and the other engine's map dropped with `kit_texture_not_used`, both re-laid
  by the kit's marker where the target takes them, loose configs with no `UniformParameter`
  staged on pre-Fox, TC-KIT-26's PES 17 half (TC-KIT-19, 21, 22, 23, 26, 28, 29, TC-BIN-04;
  then, lead: the pre-Fox tracer's `studio/` fixture regenerated with its `Kits/g1/`,
  `make_studio.py` `WITH_KIT`, and the parity rows for the kit config and the two kit textures
  turned Exact, comparing Red's kept absent number-texture names on the way); (b)
  `dds_compression` (the setting, every emitted DDS wrapped on PES 15-17 where the task
  finalizes its entries, already-wrapped sources passed through; the lead's level measurement
  chose 6, `wezlib::compress`'s default, so the lib is untouched: DECISIONS 2026-10-08,
  `settings.md` "DDS compression cost"; TC-TEX-08); (c) collars for pre-Fox
  (`paths::collar` and `subset::collar_file` by engine, TC-CMN-08's compile half); (d) the
  installed loose kit configs re-emitted with FPC values on pre-Fox (TC-BIN-18).
  (a) done 2026-10-08 (`paths::kit_texture(engine, name)`; `subset::kit_texture_not_compiled`,
  one function for both gate walks, so a stem outside `KIT_TEXTURE_STEMS`'s seven is named on
  pre-Fox as on Fox instead of the kit clause being dropped (the sidekick's contradiction,
  accepted); `plan::drop_other_engine_map` (`kit_srm` on pre-Fox, `kit_mask` on Fox);
  `processing/kit.rs` loops the seven stems, `{main}_mask` / `{main}_srm`, `UV_MAPPED_STEMS`
  re-laid by `relaid_texture` with `kit_layout_converted` once per kit when any own or
  inherited UV-mapped texture is re-laid (a placeholder kit's own map included, the
  placeholder and the template never: `pipeline.md` "Layout conversion"), mask and srm
  converted in the `Color` role like `kit`, the template pushed byte for byte with no decode,
  the uniparam entry `Option<Entry>` and `None` on pre-Fox; `Templates::kit_mask()`
  (`RESOURCES` 13); help; `tests/cli/prefox_kits.rs`, `kit_layout.rs` TC-KIT-28 and TC-KIT-26's
  PES 17 half with the band helper TC-KIT-18 now shares, stricter (each band must differ);
  mutants: 38, 32 caught, 6 unviable (`Default::default()` on types without one), 0 missed
  after the lead's test for the two survivors on the re-lay guard: a marked placeholder kit's
  own mask re-laid, its number atlas and the placeholder not, TC-KIT-20 in
  `prefox_kits.rs`). The lead's parity fixture task of (a) follows as its own commit.
  (b) done 2026-10-08 (`settings.rs` `DdsCompression { Auto, On, Off }` read through a serde
  visitor so a wrong value tells the member "expected true, false or "auto"" (the sidekick's
  contradiction of the untagged-helper ruling, accepted: the helper's error named a Rust type),
  `TeamCompilerSettings::compresses_dds(version)` false on Fox; `CompileContext::compress_dds`
  through `PlannedRun`; `processing::wrap_dds` on the worker right after `materialize`, the one
  place every task's entries pass, `.dds` in any case, already-wrapped entries left (portraits
  are DDS entries too and are wrapped like Red's zlib pass over the whole team tree did; no DDS
  sits in a nested face CPK); `texture::convert` on pre-Fox returns a wrapped source verbatim
  when `keeps_blocks` finds the converter's output holds the source's mip data byte for byte
  (read from the output, the converter's keep rule being private to `dds_convert`: the source
  is inflated a second time, a public predicate there would skip it, not worth a two-crate
  change for milliseconds; the lead cut the predicate's format and size conjuncts, which the
  data equality implies and which left two `&&` mutants no test could tell apart); help
  paragraph; `tests/cli/dds_compression.rs` TC-TEX-08's six runs, the PES 21
  ones without the face model (`.model` is `content_not_yet_compiled` on Fox until 4.17, the
  portrait keeps "Fox ignores the setting" observable), config errors for `1` and `"yes"`; test
  mode wraps its loose `.dds` too when the setting resolves on, by design (the setting follows
  `multicpk_mode`, not the layout), untested; mutants: 26 after the predicate cut, 19 caught, 7
  unviable (`Default::default()` on types without one, serde impls included), 0 missed)
  (c) done 2026-10-08 (`subset::collar_file(file, engine)`: the model format the engine reads
  compiles, the other native format and glTF are named until 4.17 converts them, any other
  kind passed over on both engines (a non-model file in `Collars/`, kept only with the strict
  check off, was named on pre-Fox before: now ignored as on Fox); the pre-Fox walk filters
  collars as the Fox one does; `collars::export_collar` takes the engine; `paths::collar(engine,
  id)`, pre-Fox `common/character0/model/character/uniform/nocloth/collar_NNN.model`, the
  referee-marker callers passing `Engine::Fox` (refs compile on Fox only until 4.27); the task
  writes the file unchanged, a wrapped `.model` still wrapped; the loose configs wear the ID
  through `TeamKitEdits` already; help; `tests/cli/collars.rs` TC-CMN-08's compile half, an
  `.fmdl` named on PES 17, two PES 17 claimants; the collar reaches the slots a `Midcup` export
  does not resend only through 4.16d's re-emitted loose configs, so the help promises the
  export's kits until then; mutants: 26, 21 caught, 5 unviable (`Default::default()` on types
  without one), 0 missed)
  (d) done 2026-10-08 (`WorkingBins::loose_kit_configs`, gathered on pre-Fox only by the
  working-bin walk as it passes, `installed::take_loose_kit_configs`: every `.bin` directly in a
  digits-only team folder under `paths::TEAM_KIT_CONFIGS`, nearest CPK first, unwrapped, not
  parsed; `kit_configs::loose_kit_configs(installed, uni_color, team_kits, version)` returns the
  configs a `Midcup` export's absent slots changed as `Entry`s at `paths::kit_config` plus the
  FPC findings, a `Full` export nothing; `kit_configs` and it share `absent_slots` and the
  bytes-based `edit_absent_slot`, `edit_config` now editing an `Option<Vec<u8>>` so the collar
  edit chains on the FPC edit's bytes; the writer's `None` arm of `match uniform_parameter`
  adds them through `add_bin`, so with teams parts they go to the bins CPK, in test mode under
  the test prefix, overrides applied, never `dds_compression`-wrapped (DECISIONS 2026-10-08,
  `fpc_toggle.md` "patched in place"); help; `tests/cli/bins.rs` TC-BIN-18, the collar
  reaching an absent slot's loose config, a `Full` export re-emitting nothing, the nearest CPK
  winning; the walk's cost on pre-Fox (one small read per listed CPK's team configs) is not
  measured; mutants: 43, 32 caught, 11 unviable (`Default::default()` on types without one),
  0 missed). 4.16 closed; the maintainer questions above stay open.

- [x] 4.17 **Cross-format conversion and source selection**: target-native first, then glTF, then
  the opposite native format converted through `model_convert::convert` (FMDL → `.model` + `.mtl`
  as one bundle, `.model` → FMDL), `model_conversion_failed`; `model_source_ambiguous` keeps its
  catalog row but its only input, two glTF files of one stem, is Phase 7's, so no scenario here; a
  selected glTF representation refused with `model_gltf_unsupported` (folder dropped, not
  pass-through-eligible); `bone_folded_for_version`, `skeleton_retargeted`, an SKL generated from
  the IR for a converted model with bones outside the target's tables,
  `vertex_too_far_from_origin` on the source model, the `metal` family's environment-cubemap
  sampler and template on pre-Fox (lead first: the cubemap as a lead-authored fixture with a
  provenance README). Plan: `pipeline.md` step 3 "Format conversion", "Resolved decisions" (Model
  source selection); `development_plan.md` "Phase 4" `processing/` (glTF refusal);
  `model_conversion/README.md`; `conversion.md` "Cost" (a same-engine model of another version
  also takes the IR round trip when the pre-check finds a changed bone). IDs: TC-MOD-26..30,
  TC-MOD-34, TC-MOD-36, TC-CMN-09. Crates: tc (`plan/subset.rs`, `plan/mod.rs`,
  `processing/model.rs`, `processing/prefox_face.rs`, `processing/mod.rs`, `messages.rs`,
  `templates.rs`) → verify: the tracer's `fcl_hair.fmdl` compiled for PES 17 yields a `.model` +
  `.mtl` pair in the face CPK that `pes_model` reads back with the FMDL's mesh count (anti-blur
  meshes may change the count: the red run measures both, and the acceptance text is corrected
  to the measured relation if so); `boots.fmdl` beside `boots.model` compiles the FMDL on PES 21
  and the `.model` on PES 17, no conversion finding either way; `boots.glb` alone on PES 21
  reports `model_gltf_unsupported` and drops the folder. Also, from 4.9b: a collar in the other
  engine's format converted (TC-CMN-09; an FMDL collar for pre-Fox gets the stock collars'
  material names; a `.model` collar for Fox has no `.mtl` to convert with: ruled at slice e).
  Also, from 4.18: a `.model` source with hand weights compiled for Fox is converted, then
  split. Owns the mapping of `model_convert`'s loss codes to findings (the hand-split round
  trips drop them "until cross-format conversion maps them") and the other format in shared
  `Faces/`, `Boots/`, `Gloves/` folders, `Common/` models and `.common` links (4.20 presumes
  it). Recon: `.tmp/4_17/recon_4_17.md`. Rulings: DECISIONS 2026-10-08 "Cross-format source
  selection" (per stem; the other engine's companions ignored silently with their model; the
  paired `.skl` the conversion's bind pose; the converted `.mtl` packed as `<stem>.mtl`; the
  generated SKL on a member's `.skl` path; convert, then split).
  Slices: (lead) done 2026-10-08: the environment cubemap bundled (`resources/templates/env.dds`,
  the FBM template's, byte-identical to the working referee exports'; README), the rulings, this
  text; (a) Fox → pre-Fox in the face task: a player folder's `.fmdl` selected on PES 15-17 when
  no `.model` of its stem exists, converted with its paired `.skl`, packed with its converted
  `.mtl` as `<stem>.mtl`, `fcl_hair_sim.fclo` ignored, TC-MOD-27 on the tracer; (b) pre-Fox →
  Fox in the Models task and the selection rule on both engines, the generated SKL as the
  slot's skeleton, `model_conversion_failed`, the converted form checked (TC-MOD-26, 29, 30,
  34); (c) the loss codes mapped (catalog rows lead-first) and reported by the hand splits too,
  a selected glTF refused with `model_gltf_unsupported` (TC-MOD-28); (d) `metal` on pre-Fox:
  `Templates::environment_map()`, the `EnvironmentMap` sampler added to a converted `Metal`
  material, the cubemap emitted into the folder's texture home (TC-MOD-36); (e) collars across
  engines (TC-CMN-09; the `.model` → Fox and glTF collar rulings made then); (f) shared folders,
  `Common/` models and `.common` links in the other format, in three: (f1) the gates lifted
  where the conversion already runs (a shared folder's `.model` on Fox, a shared `Faces/`
  FMDL and its `.skl` on pre-Fox, with a test of a shared metal FMDL's environment map) and a
  shared folder's glTF dropped at planning with its linking players; (f2) a shared `Boots/`
  or `Gloves/` folder's FMDL and an `ingame_face` player's FMDL parts converted by the
  pre-Fox boots and gloves writer; (f3) `Common/` models and `.common` links in the other
  format and the leftovers, in three (exploration at this HEAD in
  `.tmp/4_17/sk_4_17f3_explore.md`): (f3a) the leftovers, ruled in DECISIONS 2026-10-08 "An
  `env` texture link, one sampler shape, every `model` context alike, no kit warning on a
  folder the glTF drop orphaned" (a texture link of stem `env` counting as the folder's
  `env`; the writer's `add_environment_map` unconditional as the face's; every `model`
  context of a folder's task through `source_name`, the collar's the file name; the glTF
  drop removing the shared folders it orphans; a probe of a pre-Fox per-kit FMDL set under
  `kit_variant_model_messages`, which has no engine check); (f3b) PES 15-17: an FMDL in
  `Common/` with its `.skl`, converted once in the Common models task into
  `<packed>.model` + `<stem>.mtl` pointed at the Common textures, an `x.fmdl.common` link
  given `PreFoxCommonModel` (a `PreFoxPart` under the marker) and listed at the Common
  directory with that `.mtl`, the deep pairing of such a link; (f3c) PES 18-21: a `.model`
  + `.mtl` in `Common/` linked by `x.model.common`, converted in each linking player's
  Models task (baked, as a Common FMDL is) with the `.mtl` found in `Common/`, per-kit
  `.model` sets counted by `model_variant_sets` (`kit_variant_model_fox` on them, the lowest
  converted), and a converted `.mtl` path into the pre-Fox team Common directory
  (`model/character/uniform/common/<team>/x.dds`) pointed at the Fox Common texture
  directory when `Common/` holds the stem (today kept verbatim, the deep check calling it
  supplied); (g) the same-engine pre-check, in three slices: (g1, done 2026-10-09) the
  measurement of which pose each real model is bound to (`.tmp/4_17/sk_4_17g1_report.md`: the
  games' own files and the tracers), which withdrew the 2026-10-08 ruling (faces carry a
  per-face `skf_*` pose, boots a boots pose 7° off the body on every version, the tracer gloves
  PES 15's arm pose moved as one piece); (g2) `model_convert`: `needs_conversion` tests
  whether any vertex blends bones whose deltas to the target differ (DECISIONS 2026-10-09),
  `skf_*` out of the check and the re-bind, the tolerance 3e-3, the fixtures the lead copied
  into `model_convert/tests/fixtures/` (stock PES 17 boots and glove, stock PES 21 boots with
  its `.skl`, the pre-Fox tracer's three models) never flagged on their own version; (g3a,
  done 2026-10-09) `team_compiler`, Fox: every selected FMDL of the Models task (a player's, a
  shared folder's, a `.common` link's) pre-checked with its paired `.skl` as the bind pose and
  moved onto the version's skeleton when flagged, the gloves task reading a hand-split part's
  `.skl` for its re-conversion, the skeleton a conversion writes for a slotless role dropped
  with no finding, collars not pre-checked (DECISIONS 2026-10-09, two entries), `pipeline.md`
  step 3's pre-check sentence rewritten, TC-MOD-46 to 48; (g3b, done 2026-10-09)
  `team_compiler`, pre-Fox: every `.model` the face, the shared boots and gloves writer (the
  `ingame_face` parts included) and the Common models task pack pre-checked and moved when
  flagged, the member's `.mtl` packed beside it, the Common `.mtl` a link names read by the
  face task (which also ends the 2026-10-08 refusal of a hand split over a Common `.mtl`), a
  `Common/` `.model` with no `.mtl` packed as it is; not pre-checked: a member's own `face.xml`'s
  models and `.model` collars (DECISIONS 2026-10-09, three entries); TC-MOD-49 to 52 on the
  stock cap model, the one pre-Fox fixture flagged (PES 15 folds two of its bones;
  `.tmp/4_17/lead_g3a_probe/precheck_prefox.txt`).
  Slice c was split: c1 the loss codes and the pre-check
  measurement, c2 the selected glTF (`model_gltf_unsupported`, TC-MOD-28).
  Open for converge (design health): `output/writer.rs` `commit_folder` takes an empty
  textures batch for a failed task's (`TaskBatch::entries`'s doc, "empty when the task
  failed") and commits nothing of the folder, an implicit rule planning honors since f3a by
  never planning a textures task with nothing to emit; a `failed` flag on `TaskBatch`, or
  an assertion, would make it explicit (found at f3a: a planned empty task lost a whole
  face CPK with exit 0 and no finding). Also for converge (`model_convert`, found at 4.17g1-g2): the FMDL importer
  drops Konami's `bone_matrices` block as `native_field_dropped` and takes the PES 21 tables
  as the bind pose, while 4.17g1 verified (to 1e-7 on body bones) that the block holds the
  bind pose; a Konami face converted for PES 15-17 so loses its per-face `skf_*` pose (up to
  0.34). The importer should read the block as the bind pose when present, and the exporter,
  which writes `bone_matrices: None`, should write it from the IR's bone matrices as Konami
  does, after which `needs_skl` need not gate on a `skf_*` bone (the pose then travels in the
  FMDL, and a converted face writes no `.skl`; the lead test
  `a_converted_face_carries_its_own_skf_pose_in_the_skl_it_writes` moves to the block); until
  then the compiler drops a face's conversion-written `.skl` silently (DECISIONS 2026-10-09).
  Whether the Fox game reads the block or the face diff for a face's pose is
  `docs/QUESTIONS.md` "Face diffs are engine-specific". Also for converge (`model_convert`
  and the collar task, found at 4.17g3b): a `.model` collar posed off the target's skeleton
  is packed as it is (DECISIONS 2026-10-09), because the IR import refuses a model whose
  material the set lacks and a collar's materials are the game's `uniform.mtl`; the stock
  PES 17 collar is flagged on PES 15 (`.tmp/4_17/lead_g3a_probe/precheck_prefox.txt`), so a
  real case waits on the import taking a `.model` without its set (its materials kept by name,
  which the collar task renames to the stock names anyway). Also for converge
  (`model_convert`, found at 4.17g3b): the IR carries no `.model` mesh tags (`Mesh::tags`), so
  every conversion of a `.model`, a same-engine move included, drops them as
  `native_field_dropped (field=tags)`; the stock cap's tag is `Captainmark`. Whether the game
  reads a mesh tag is unknown; if it does, the IR needs the field. Also for converge (maintainer, 2026-10-08):
  `team_compiler` has outgrown mutation runs: 2,171 mutants (measured 2026-10-08 with
  `cargo mutants --list`), about 10 s each on the PC, so a whole-crate run is about six
  hours, twice per phase close, and local-only since 4.14d because the crate's test
  binaries link egui (the direct `egui` dependency and `studio_core`) and that build hit
  the VPS cap. Two measures, the first the larger: (1) investigate taking egui out of the
  test binaries' link, by moving the compiler's view (`view/`, `gui_run.rs`, about 450
  lines) into its own crate or behind a feature the mutation run turns off, and checking
  how `studio_core`'s egui parts are gated; if the link goes, the crate is VPS-eligible
  again and every whole-crate run splits in half; (2) move the self-contained modules to
  lib crates even with the compiler as their only consumer, their tests with them (a lib's
  mutants run the lib's tests alone): `face_diff` (32 mutants), `user_face_xml` (93),
  `kit_variants` (11), `mtl_search` (18), 7% of the crate, so for design health rather than
  run time; `user_face_xml` after 4.20 changes it. Each is a crate-boundary move, a plan
  edit and a decision entry at that point. From f3b, for 4.20 or converge: a member's own
  `face.xml` naming a converted Common model (`common/XXX/legs.mtl`, `oral_legs_*.model`
  from `legs.fmdl`) gets `xml_model_not_found`, the deep pass resolving Common references
  among source files by kind; a Common FMDL a `.model` of its stem beats is still
  deep-checked (`model_broken`, `DropFile`) where a player folder's beaten FMDL is unread;
  the Common models task never hand-splits (a Common `.model` is not split either); two
  converted parts merged under the marker with a material of one name but different
  texture places fail `merge_material_conflict`, which the merge rules do not cover. From
  f3c, for 4.20 or converge: on Fox a player's own `.mtl` override for a Common `.model`
  (`legs.mtl` beside `legs.model.common`) is looked up by the deep pass against the
  folder's textures while the Models task points the part's paths among `Common/`'s alone,
  so a texture only the player holds counts as supplied and stays `./face.dds` in the FMDL
  with no finding (`player_folders.md` says the override layers on top, not where its
  textures resolve: a plan ruling); a pre-Fox Common path an installed CPK supplies
  (`texture_supply` reads them) is called supplied by the deep pass but left as written,
  since the path is pointed only when `Common/` holds the stem (pointing it whenever the
  deep pass calls it supplied would close it); an unlinked Common `.model` or `.mtl` is
  still deep-checked and dropped when broken, as an unlinked Common FMDL is, where the gate
  and the plan sentence say "ignored"; two links in one folder to one Common model under
  two roles bring it in once under the first link's role, `common_material` following
  (pre-existing, the plan silent).
  Open for the maintainer (`docs/QUESTIONS.md` "Converted collars", "`dummy_kit` on the
  modded PES 15-17 exes"). Open for a
  later slice: a converted FMDL in a folder with the member's own `face.xml` is silently not
  compiled (the xml's references, `xml_model_unlisted`, `XmlFace::pack` and the
  `model_material_undefined` comparison know `.model` files only; a plan ruling on how an xml
  names a converted model and its `<stem>.mtl` comes first; analysis in
  `.tmp/4_17/sk_4_17a_report.md` R3); a converted material whose base is the game's `dummy_bsm`
  (Konami's `addon_oral.fmdl`) keeps its Fox path on pre-Fox
  (`/Assets/pes16/model/character/common/sourceimages/dummy_bsm.dds`), where the legacy
  converter wrote `./.dds`, a directory with the name dropped (measured at 4.17c1 on
  `legacy19to16_oral.mtl`): neither resolves on PES 15-17, so the game draws that material
  with its fallback either way, and the same model's hidden oral mesh (`invisible`, a flag a
  `.mtl` cannot express, now the Warning `mesh_flags_dropped`) shows on PES 17; open for the
  maintainer (`docs/QUESTIONS.md` "Hidden Fox meshes converted to pre-Fox"). From 4.17b:
  the Warnings and Infos the target format's check fires on a converted form are not reported
  (rule with slice c: Warnings kept with their code, Infos dropped); the same-engine
  `needs_conversion` pre-check of a selected native model is not run yet (slice c); per-kit
  `.model` variants on Fox are not a set (`kit_variants::model_variant_sets` reads FMDLs only,
  so `pants_kit1.model` + `pants_kit2.model` both convert and merge with no
  `kit_variant_model_fox`; slice f); a role-less `.model` on Fox (in `gloves/` naming no hand,
  in `common/` under a lenient file-type check) is skipped silently by the gate, 4.17a's FMDL
  gap mirrored, until 4.20; a converted `.mtl` path into the pre-Fox team Common directory
  (`model/character/uniform/common/<team>/x.dds`) on Fox is called supplied by the deep check
  when `Common/` or an installed CPK holds its stem, but the Fox Models task points converted
  paths at the folder's own textures and its links only, so the packed FMDL keeps the pre-Fox
  directory with no finding (slice f, with `Common/`). From 4.17d: the converter's
  `EnvironmentMap` sampler (`to_prefox::sampler_for_role`) writes no `mipfilter`, where the
  legacy FBM template's `fbm.mtl` writes `mipfilter="linear"` beside the same settings (a
  `model_convert` question, with slice g's poses). GPT review
  (c) queued: the two-crate 4.17a change (`model_convert` FMDL import's dummy rule,
  `team_compiler` conversion), sections `pipeline.md` step 3 "Format conversion", DECISIONS
  2026-10-08 (both 4.17 entries). GPT review (d) queued: the two-crate 4.17d change
  (`model_convert` re-exporting `from_fox_shader`; `team_compiler` the deep pass's metal
  record, planning's `environment_map` flag, the textures task's template entry, the face
  task's sampler), sections `pipeline.md` step 6 (the last two paragraphs),
  `model_format.md` the `metal` and `environment` rows, TC-MOD-36, DECISIONS 2026-10-08
  "environment cubemap".
  (a) done 2026-10-08 (`subset.rs`: an `.fmdl` with no `.model` of its path stem in the same
  directory is `PreFoxModel` on pre-Fox (`FolderModels::pre_fox_model_stems`,
  `converted_stems`), its paired `.skl` the new `PlayerFile::ConversionSkeleton`, every
  exhaustive match extended; the gate's player walk skips a role-less `.fmdl`, `.skl` or
  `.fclo` (commented as going with the gate at 4.20); an FMDL under `ingame_face` or in a
  shared folder stays named (only the face converts); `TaskKind::files` lists the skeleton for
  the face; new `processing/conversion.rs` `fmdl_for_pre_fox` (`convert` on a Fox bundle,
  `model_conversion_failed` on error, `bone_folded_for_version` and `skeleton_retargeted`
  reported with context `bone`/`bones`, the other loss codes dropped until slice c);
  `prefox_face.rs` `FaceSource { Member, Converted }` so a converted `.model` never pairs a
  member's `.mtl`, the set pointed by `point_materials` then `point_reserved_kit_stems`
  (`dummy_kit*` to the team's Common directory) and packed as `<stem>.mtl`; `model_convert`
  `is_game_dummy`: the FMDL import gives the game's `dummy_nrm`/`dummy_srm` no role (they stay
  native samplers, a Fox round trip writes them back; `ir_to_model` reports no
  `material_texture_unused` for them), so the oral legacy-reference test now matches the
  legacy's `Basic_C` and sampler set; help; `tests/cli/conversion.rs` TC-MOD-27 (the tracer on
  PES 17: four `.model` + `.mtl` pairs, FMDL meshes 3/3/1/1 to `.model` meshes 2/2/1/1, the
  anti-blur mesh folded back, the `kit` material `Basic_C` with `DiffuseMap` alone at the team
  Common `dummy_kit.dds`, no `/Assets/` path left), the PES 15 run reporting
  `skeleton_retargeted bones=7` for the hair and the boots, a beaten FMDL silent, a corrupt
  `.skl` failing the folder; `compile.rs` and `sideload.rs` flipped; one rework round (the
  lead's ruling 5 premise wrong: the converted `.mtl` carried the Fox dummy paths; the
  sidekick's import rule replaced the lead's, which broke Fox round trips); mutants: 92
  (model_convert 17, team_compiler 75), 79 caught, 13 unviable (`Default::default()` on types
  without one), 0 missed; `just bindings` green).
  (b) done 2026-10-08 (`subset.rs`: on Fox a `.model` with no `.fmdl` of its path stem is
  `PlayerFile::Model` through `model_role` (`FolderModels::native_model_stems`, the renamed
  `pre_fox_model_stems`, now both engines' native stems; `FolderModels::beaten`), every `.mtl`
  outside `common/` is `PlayerFile::Material` on both engines, the gate skips a role-less
  `.model` or `.mtl` (with the gate at 4.20); `plan/mod.rs` `TaskKind::files` reads a source's
  `.mtl` files on Fox only when the package converts a `.model`; `processing/conversion.rs`
  `FoxConversion { model, skeleton }`, `model_for_fox(name, bytes, mtl, ctx, findings)`, and
  `target_form_failure` run by both directions on the converted form (a far vertex
  `vertex_too_far_from_origin` with context `model`, `count`; any other Error
  `model_conversion_failed` with the rule's code; Warnings and Infos not reported, slice c);
  `processing/model.rs` `package` converts a selected `.model` with the `.mtl` `mtl_for` finds
  among its source's files, the generated skeleton as the part's (equal to a member's, else
  `skl_merge_conflict`; `skl_no_slot` with context `model` for a slotless role, a glove
  included), the gloves task re-converting a hand-split part without re-reporting; `deep/`:
  `pairings` on Fox for a selected `.model`, `unread` skips a beaten model on both engines and,
  on Fox, a `.mtl` no selected `.model` pairs with, `held` on Fox too so a selected `.model`'s
  `.mtl` gets `mtl_texture_not_found`; `Fired`, `Fired::fox`, `Fired::pre_fox`, `summed`
  `pub(crate)`; help; `tests/cli/conversion.rs` TC-MOD-26 (both engines, whole finding lists,
  the beaten model silent, a far beaten `.model` dropping nothing on PES 21), TC-MOD-29 (the
  boots package left out, the blank face and `skin.ftex` committed, with and without
  `pass_through`), TC-MOD-30, TC-MOD-34 (card head 1 mesh to 2 FMDL meshes, one anti-blur;
  `skin.dds` in the table under the player's home, `skin.ftex` in the CPK; the bundled
  `boots.skl`), the hand split of a converted `.model` (8/8/24 of 40 faces), the generated
  skeleton alone, equal to a member's and in conflict, `skl_no_slot` on `face_high.model`, the
  Fox `.mtl` texture Warning; six contradictions accepted (four brief errors, two measured facts
  that corrected TC-MOD-34 and TC-MOD-29 plus three catalog rows, DECISIONS 2026-10-08 "A
  failed conversion leaves its package out"); one rework round for two plan gaps the sidekick
  found (the Fox `.mtl` texture check, the beaten model deep-checked), the rest parked above;
  mutants: 108, 91 caught, 17 unviable (`Default::default()` on types without one, one `||`
  in a `let` chain), 0 missed).
  (c1) done 2026-10-08 (`conversion.rs` `reported` maps every code `model_convert::loss`
  documents (16) to a catalog row, context `model`, then the subject's `mesh`/`material`/`bone`
  index, then the detail under the row's key (`name`, `count`, `parameter`, `sampler`,
  `texture`, `field`; the folds' `bone` detail in the index's place), `native_field_dropped`
  for `invisible`/`no_shadow_cast` as the Warning `mesh_flags_dropped`; `messages.rs` 15 `Code`
  variants (`material_texture_unused`'s had no variant), `ALL` 128; help; the tracer tests on
  PES 17 and 15 pin the measured loss lines (`bone_matrices` on every model, the hair's two
  `skl_parent`, the `shirt` material's `no_shadow_cast` on the boots and the hair), the hand
  split's two `dummy_texture_added`, the card's three losses in the unit test; the same-engine
  pre-check was wired on both engines as the plan said, measured, and taken out again: the
  sidekick removed the pre-Fox half itself when the parity test failed (every tracer model
  true on PES 17), the lead's rework unwired the Fox half (true only for boots shipping the
  game's `boots.skl`), the drafts kept under `.tmp/` for slice (g) (DECISIONS 2026-10-08 "The
  same-engine pre-check waits"); the `.expect` a `.model`'s Common `.mtl` seemed to reach on Fox
  is unreachable (the plain search never looks in `Common/`, and the gate names a Common
  `.mtl` and a `.common` link to one), measured by a probe, no change; `dummy_bsm` measured
  (open item above); one rework round; mutants: 23, 21 caught, 2 unviable (`Default::default()`
  on `Code`, one `||` in a `let` chain), 0 missed).
  (c2) done 2026-10-08 (`subset.rs`: `PlayerFile::UnsupportedGltf` for a `.glb`/`.gltf` with
  no target-native model of its path stem beside it, at any position; `FolderModels::gltf_stems`
  and `beaten` widened to the plan's order, target-native first, then glTF, then the other
  engine's format (a glTF beats the opposite native model, is beaten by the target's own);
  the player gates skip a beaten glTF, the shared gates still name a shared folder's glTF
  (slice f); `plan/mod.rs` `drop_gltf_folders` right after `drop_other_engine_map`: one
  `model_gltf_unsupported` (E, `DropFolder`, context `file` below the folder as
  `xml_ignored_fox` names its file) per selected glTF of a mapped player folder, the folder's
  roster slots removed as validation removes a dropped folder's, so it plans no task and
  `check` never reports it; `Code::ModelGltfUnsupported`; help; `tests/cli/conversion.rs`
  TC-MOD-28 (the first export's slot 07 added so a CPK is written and the absence of slot 05
  proves something; `check` reports `export_identified` alone on both); unit tests for the
  roles on both engines, `beaten` with glTF cases, the gate flips, the planning drop for a
  team and a referee export; three contradictions accepted (the relative file name, a lead
  fix; the role at any position; the pre-Fox gate's skip, a brief omission); measured: at
  HEAD the gate named a `boots.glb` beside `boots.fmdl` too, now ignored; parked: a shared
  folder linked only by a dropped folder could still report `kit_variant_model_fox` (no test
  reaches it); mutants: 77, 69 caught, 8 unviable (`Default::default()` on types without
  one), 0 missed).
  (d) done 2026-10-08 (`templates.rs`: `ENVIRONMENT_MAP` (`env.dds`, 14 resources, the
  override reported as every other's) and `environment_map()`; `deep/model.rs`
  `ModelRead::metal`, an FMDL with a material the converter's own rule reads as metal
  (`model_convert::materials::from_fox_shader`, re-exported for it: a lead fix over the
  sidekick's `ggx` substring, which read a `glass`+`ggx` shader as metal where the conversion
  makes it glass); `deep/mod.rs` `ContentPass::metal_models`, the paths recorded whatever the
  target as `hand_weighted` is (contradiction accepted: the pass does not know whose face
  takes a file; the sidekick's Fox clear dropped as a second gate, a lead fix); carried
  through `CheckedSource` and `ExportToPlan` to `plan/mod.rs` `converts_metal`: a folder
  with no own `face.xml` whose `PlayerFile::PreFoxModel` file is among them gets
  `ModelFolder::environment_map`, and `folder_tasks` plans its textures task even with no
  texture; `processing/texture.rs` `folder_textures` chains `("env", template)` unless a
  source's `Texture` role folds to `env`; `processing/prefox_face.rs`
  `add_environment_map(set, home)` before `point_materials` (contradiction accepted: the
  pointing respells the path as the folder spells its own `env` texture), the sampler after
  the material's last sampler with `sampler_for_role(Environment)`'s settings copied by hand
  (the converter's `to_prefox` is crate-private; `SamplerSettings` is its own type, so a
  re-export would still need a mapping); help; `tests/cli/conversion.rs` TC-MOD-36 (the
  template's bytes under the player's home, zlibbed, the sampler's settings and place, the
  vectors), the member's own `env.dds` (written WESYS-wrapped, since a plain DDS is
  re-encoded), the `templates/env.dds` override, TC-MOD-27's test asserting no sampler and
  no `env.dds` for the tracer; unit tests in `deep` (metal on both targets, a glass+ggx
  shader not), `plan` (slot 05 flagged with a fileless textures task in the face's group,
  a beaten FMDL and an own-xml folder not, PES 21 nothing) and `prefox_face` (one sampler
  added, one left, the respelling); three contradictions accepted, the `mipfilter` one
  parked above; mutants: see the 4.17d log line).
  (e) done 2026-10-08 (lead first, `4431669`: the plan's "Collars" rewritten with the
  rulings, the two message rows, TC-CMN-09's text, DECISIONS "Collars across engines".
  `plan/subset.rs` `CollarFile`: an FMDL `Compiled` on both engines, a `.model` on Fox
  `NotCompiled` (the gate names it: no `.mtl` to convert its materials with until the
  templates ship a stock `uniform.mtl`), a glTF the new `Unsupported`; the pre-Fox gate's
  collar clause removed (nothing is `NotCompiled` on pre-Fox); `plan/collars.rs`
  `export_collar`: an `Unsupported` file reports `model_gltf_unsupported` (`Scope::File`,
  `DropFile`, context `file`) and claims nothing; `processing/conversion.rs`
  `PreFoxMaterials { Converted, StockCollar }` as `fmdl_for_pre_fox`'s last parameter:
  `StockCollar` renames the converted form's materials (`stock_collar_materials`: the first
  `uni_collar`, every other `uni_shirts`, the list collapsed to those two, each mesh pointed
  at its name) before the target-form check, and reports no loss whose subject is a
  material (`PreFoxMaterials::reports`: the `.mtl` is not written); `processing/mod.rs` the
  collar arm converts an FMDL on PES 15-17 with no skeleton (the version's body table) and
  writes the `.model` alone, a failure the existing `DropFile` arm; help. Tests:
  `tests/cli/collars.rs` TC-CMN-09 (the tracer's boots, whose antiblur mesh the conversion
  folds first: `collar_012.model` under `nocloth` with `uni_collar` and `uni_shirts`, the
  meshes pointed as the decoded source's, no `.fmdl` or `.mtl`, the kits wearing 12, the
  deep pass's and the conversion's model-level findings pinned), the glTF collar on both
  engines (dropped, the kits keeping their 30 and 31, exit 1), the failed conversion (a
  control character in a material name; the kits wear the game's own collar 12, planning
  having pointed them before the task ran: ruled, DECISIONS "A dropped collar's kits"), the
  `.model` collar on PES 21 still named; unit tests for `collar_file`'s table, the gate
  walks, `export_collar`'s drop, the rename, the loss filter (the tracer's boots converted
  both ways). Three contradictions accepted (the dead gate clause, deleted at rework; the
  brief's failing-model pointer, a `.model` one; the brief's kit-assertion pointer); plan
  gaps: the dropped collar's kits (documented), the material losses (fixed at rework), the
  converter's flag-split part counted as another material (documented, a `model_convert`
  provenance change if ever needed); one rework round; mutants: see the 4.17e log line).
  (f1) done 2026-10-08 (lead first, `de62e60`: the plan's step 3 "Format conversion" on
  shared folders, the `model_gltf_unsupported` row, DECISIONS "A shared folder in the other
  format", the slice list f1/f2/f3; exploration in `.tmp/4_17/sk_4_17f_explore.md`.
  `plan/subset.rs` `shared_not_compiled`: a shared folder's `.model` and `.mtl` no longer
  named on Fox (the Models task converts them as a player's; a `Material` role exempt from
  the other-package check, `PlayerFile::package` giving a `.mtl` its pre-Fox answer), a
  beaten model skipped as the player gates skip it (contradiction accepted: without it the
  shared gate named a glTF an FMDL beats), `UnsupportedGltf` never met;
  `pre_fox_shared_not_compiled`: a `Faces/` folder's FMDL and `ConversionSkeleton` no longer
  named (the face converts them with the skeleton as bind pose), a `Boots/`/`Gloves/` FMDL
  still named (f2: lifting it by hand panics in `prefox_shared`'s `.mtl` lookup, measured),
  a beaten model skipped, a selected glTF skipped; `plan/mod.rs` `drop_gltf_folders`: every
  shared folder holding a selected glTF removed from the export, each mapped player linking
  it (and holding no glTF of his own) dropped with one `model_gltf_unsupported` per shared
  glTF, `file` the shared file's export path; help. Tests: `tests/cli/conversion.rs` a
  shared boots `.model` + `.mtl` converted to the shared package's `boots.fmdl` on PES 21
  (mesh count, `boots.skl`, `skin.ftex`, paths pointed at the shared output), a shared
  folder's glTF dropping its linking player on PES 21 and 17 with `check` silent, a linked
  face's metal FMDL reflecting the environment map in the player's home;
  `tests/cli/prefox_faces.rs` a linked face's FMDL (the tracer's hair with its `.skl` and
  texture) converted into the player's face CPK (`oral_hat_win32.model`, `hat.mtl` pointed
  at the player's home, the `face.xml` entry, the `skl_parent` drops proving the skeleton
  read), a linked boots FMDL still named; unit gate tests on both engines, the planning
  drop. One contradiction accepted (the beaten skip); plan gaps to f2: other role-less Fox
  files in a shared folder on pre-Fox (`.skl` of a beaten FMDL, `fcl_hair_sim.fclo`) still
  named, a shared source's conversion findings naming it by its bare name on the player
  (`model=hat.fmdl` for `Faces/Round/hat.fmdl`); mutants: see the 4.17f1 log line).
  (f2) done 2026-10-08 (DECISIONS "The pre-Fox boots and gloves writer converts as the face
  does"; the plan's step 3 "Format conversion" and the `model_conversion_failed`,
  `bone_folded_for_version`, `skeleton_retargeted` rows. `plan/subset.rs`
  `FolderModels::of_player_files`: `converted_stems` filled under the marker too (an FMDL
  with a part role, so a per-kit FMDL's `.skl` gets no role; the face still unset there),
  `compiled_under_ingame_face` true for `ConversionSkeleton`, `ignored_without_role` (a
  beaten model, a role-less `.fmdl`/`.skl`/`.fclo`) shared by the player gate and the shared
  gate, `pre_fox_shared_not_compiled` accepting a boots or gloves folder's FMDL and its
  skeleton; `plan/mod.rs` `TaskKind::files` reading a `ConversionSkeleton` for its source
  package or the folder's own (contradiction accepted: a marked player's own files' source
  package is the boots, so the brief's rule would have converted his glove with no bind pose
  and nothing reporting it), `converts_metal` counting a `PreFoxPart` of the boots or gloves,
  the shared `ModelFolder` flagged too (the shared texture home is `./`, so the flag alone
  emits `env.dds` beside the models); `processing/conversion.rs` `source_name` (below the
  task's folder when in it, by segments, else the export path), used by the face, the Fox
  Models task and the writer; `processing/prefox_shared.rs` `SourceModel::skeleton`,
  `converts`, `packed_name`, a `convert` closure (`fmdl_for_pre_fox` with the source's
  skeleton, then `add_environment_map` under the flag, `point_materials`,
  `point_reserved_kit_stems`, the three made `pub(super)`), the boots' sets indexed by part,
  a converted glove packed as `<stem>.model` + `<stem>.mtl` lowercased and listed as such;
  help. Tests: `tests/cli/prefox_faces.rs` a linked boots FMDL (the tracer's hair with its
  `.skl`) converted into `boots.model` + `boots.mtl` with `./shirt.dds`, the `skl_parent`
  drops proving the skeleton read, no `.fmdl`/`.skl` in the CPK; a linked gloves folder's
  two FMDLs converted and listed in its `glove.xml`; the linked face's `model=` re-pinned to
  `Faces/Round/hat.fmdl`; `tests/cli/prefox_ingame_face.rs` a marked player's `boots.fmdl`
  and `glove_r.fmdl` converted into his own boots and gloves, paths pointed at his home and
  the reserved kit stem at the team's Common directory; `tests/cli/conversion.rs` a shared
  boots folder's metal FMDL reflecting `./env.dds` beside its models; unit tests on the
  roles under the marker, both gates, the skeleton read, the flag, `source_name`. One
  contradiction accepted (the skeleton rule), two brief errors (test 1's naming is below the
  shared folder, its task's own; `clean_model()`'s `skx_` bones as a glove add 20 warnings)
  substituted right; plan gaps: the other `model` contexts and the env-sampler asymmetry (to
  f3), a member's `.mtl` colliding with a converted glove's set (an error, ruled in
  DECISIONS), shared `Faces/` role-less Fox files ignored too (accepted); lead fixes: none;
  mutants: see the 4.17f2 log line).
  (f3b) done 2026-10-08 (lead first, `b63dd94`: DECISIONS "A Common FMDL converts once in the
  Common models task on PES 15-17", the plan's step 7 Common sentences and step 6 Common
  sentence, `player_folders.md`'s marker sentence. `plan/subset.rs` `pre_fox_common_model`
  (the Common model a link loads: the named file, or the `.model` of its stem that beats a
  Common FMDL), `pre_fox_link` and `linked_pre_fox_model` taking an FMDL link
  (`PreFoxCommonModel`; a `PreFoxPart` under the marker), `common_file_compiled` true for a
  Common FMDL and `.skl` on both engines (a beaten FMDL and a stray `.skl` ignored as a
  player's); `plan/mod.rs` `common_model_files` (the `.model` and `.mtl` files, each
  converted FMDL with the `.skl` of its stem), `CommonModel` for an FMDL link with the Common
  `.skl` and no `.mtl` (contradiction accepted: the brief's "skeleton when the slot has one"
  gave a marked link none), `roles()` pushing it as a `ConversionSkeleton`,
  `ModelFolder::common_files` holding the Common models too (contradiction accepted: the
  face needs the same stem selection), `TaskKind::CommonTextures::environment_map` (a
  converted Common FMDL in `metal_models`, no Common `env`; the task planned for it alone);
  `deep/mod.rs` `links_common_fmdl` (a link loading a Common FMDL pairs no `.mtl`);
  `processing/prefox_common.rs` converting each FMDL (`fmdl_for_pre_fox` with the Common
  skeleton, `add_environment_map` at the Common directory, `point_materials`,
  `point_reserved_kit_stems`), packed as `oral_<stem>_win32.model` + `<stem>.mtl`, the
  duplicate check case-folded (contradiction accepted: the game's file system folds),
  `folder_pack_failed` at `Common` for a member's `.mtl` of that name; `prefox_face.rs`
  `FaceSource::CommonConversion` (the entry's material at the Common directory, nothing
  packed, no `.mtl` layering); `texture.rs` `common_textures` emitting the template; help.
  Tests: `tests/cli/prefox_faces.rs` a Common FMDL (the tracer's hair with its `.skl`)
  converted once into the Common output and listed by slot 05's `face.xml` at the Common
  path with `legs.mtl`, the `skl_parent` drops on `Common`; a Common `Legs.mtl` beside it
  failing the task, the face and textures still committed; `tests/cli/conversion.rs` the
  template in the Common output, or `Common/env.dds` instead; `tests/cli/prefox_ingame_face.rs`
  a marked player's `legs.fmdl.common` converted into his boots with his own `boots.fmdl`
  (contradiction accepted: the brief's `card_model()` part cannot merge with the hair,
  `skl_merge_conflict` on `sk_head`); unit tests on the roles, the gate, the stem selection,
  the task files, the Common flag, the deep pairing. Four contradictions accepted; plan gaps
  parked (open items below): a member's own `face.xml` naming a converted Common model, a
  beaten Common FMDL still deep-checked, no hand split in the Common task, two converted
  parts' material clash under the marker; lead fixes: none; mutants: see the 4.17f3b log
  line).
  (f3c) done 2026-10-08 (lead first, `2d94071`: DECISIONS "A Common `.model` converts in
  each linking player's Models task on PES 18-21; per-kit sets span both formats; a pre-Fox
  Common texture path is pointed at the Fox Common directory", the plan's step 3 sentences
  and "Kit-dependent assets". `plan/subset.rs` `linked_model` (one function for both
  engines, `linked_fmdl` and `linked_pre_fox_model` folded), `selected_common_model(common,
  linked, engine)` (target-native first on either engine: on Fox an FMDL of the stem beats
  a linked `.model`), `common_file_compiled(file)` with no engine (both accept the same
  kinds; an unlinked Common `.model` or `.mtl` is unused on Fox); `plan/mod.rs`
  `common_models` giving a `.model` link the `.mtl` `mtl_for` finds on both engines,
  `push_common_material` (the `.mtl` as a `PlayerFile::Material`, shared with the marker
  arm), `ModelFolder::common_material`; `kit_variants::model_variant_sets` counting `.fmdl`
  and `.model` variants (a number in both formats one variant, its FMDL selected, the
  reference with the lowest variant's extension); `deep/mod.rs` pairing a Fox `.model` link
  with the Common `.mtl` its search finds, `model_material_undefined` on the player's folder
  when none, the Common `.mtl`'s texture lookup on the folder once, against `Common/`'s
  textures; `processing/model.rs` `convert_part` (the `Model` and `CommonModel` arms share
  it), a Common `.model` converted in each linking player's task with its Common `.mtl`
  (twice for two players), `point_texture(path, places, common, team_segment)` sending a
  path the deep pass reads as `Reference::Common` to the Fox Common directory when
  `Common/` holds its stem; help. Tests: `tests/cli/common_links.rs` a `legs.model.common`
  converted into slot 05's face beside his own converted card (contradiction accepted: the
  tracer's hair cannot merge with the card, `skl_merge_conflict` on `sk_head`), `skin` once
  in the Common output; two players each converting it; `tests/cli/models.rs` a
  `pants_kit1.model` + `pants_kit2.model` set, the lowest alone converted;
  `tests/cli/conversion.rs` the pre-Fox Common path pointed with `Common/shirt.dds` and left
  as written without (`mtl_texture_not_found` pinned), a linked Common `.model`'s
  `skl_no_slot` naming `Common/face_high.model` (contradiction accepted: the brief's
  `legs.model.common` name; the arm reads the Common model, never the link, and f3a and the
  plan name a converted shared source by its export path); unit tests on the roles, the
  gate, the selection on both engines, the plan, the sets, the deep pairing, the path rule.
  Plan gaps parked (open items below): a player's override `.mtl` for a Common `.model`
  resolving its textures nowhere the task points, a pre-Fox Common path an installed CPK
  supplies left as written, an unlinked Common `.model` still deep-checked, one Common
  model under two roles; lead fixes: one assertion from the mutation survivor (two files
  of one number and format stay two variants); mutants: see the 4.17f3c log line).
  (f3a) done 2026-10-08 (lead first, `a281f33`: DECISIONS "An `env` texture link, one
  sampler shape, every `model` context alike, no kit warning on a folder the glTF drop
  orphaned", the plan's step 6 paragraph and step 3 sentences, the `kit_variant_model_fox`
  and `model_conversion_failed` rows, the f3a/f3b/f3c split. `plan/mod.rs`
  `drop_gltf_folders` removing every shared folder no remaining mapped player links (no
  finding), `ModelFolder::takes_template_environment_map` (flagged and no source holds an
  `env` texture or a texture link of that stem; `ENVIRONMENT_MAP_STEM` moved here from
  `prefox_face.rs`), `folder_tasks` planning no textures task that would emit nothing
  (contradiction accepted: the brief said to plan it; built that way slot 05's whole face
  CPK vanished with exit 0 and no finding, `output/writer.rs` `commit_folder` taking an
  empty textures batch for a failed one), `folder_textures` using the method,
  `kit_variant_model_messages` given the engine and silent on pre-Fox (lead fix from the
  sidekick's probe: on PES 17 a converted `pants_kit1.fmdl` + `pants_kit2.fmdl` set was
  packed whole and listed once as `pants_kitN`, right, and still warned "lowest variant
  used"); `processing/model.rs` `skl_no_slot`, `fmdl_texture_not_found` and `parts_of` (a
  `model: &str` parameter) named through `source_name`; `prefox_face.rs` `model_name` once
  per model feeding `kit_variant_mtl_differs`, the Common-`.mtl` split failure and
  `split_face_model`; `prefox_shared.rs` `add_environment_map` unconditional as the face's,
  the comment saying why the flag is not consulted; help. Tests: the orphan removed and
  reporting nothing (and reported once when another player links it); an `env` link planning
  no textures task, the template not emitted, the sampler naming the Common `env.dds`
  (`tests/cli/conversion.rs`, the Common `env.dds` WESYS-wrapped so its bytes compare);
  a linked face folder's hand-split model named `Faces/Round/body.model`; `skl_no_slot` for
  a `face/face_high.model`; the kit-variant plan test extended with the PES 17 set (no
  warning, both variants read); `prefox_kit_variants.rs` pin corrected to
  `model=Faces/Longhair/pants_kit2.model` (contradiction accepted: the brief's premise that
  every pinned `model=` names a model directly in its folder was false there). Three
  contradictions accepted; plan gaps: the writer's implicit "empty batch = failed" rule
  (open item below), a Common part's `model` context now the link below the folder (no
  test pins it; checked at f3c, which touches that arm); mutants: see the 4.17f3a log
  line).
  Closed 2026-10-09 at converge: the criterion's three parts are TC-MOD-27's test
  (`tests/cli/conversion.rs`: the tracer's `fcl_hair.fmdl` on PES 17 as `oral_fcl_hair_win32.model`
  + `fcl_hair.mtl`, read back by `pes_model` with 2 meshes to the FMDL's 3, the anti-blur mesh
  folded into its material as the criterion allowed), TC-MOD-26's (`boots.model` beside the FMDL
  ignored on PES 21, used on PES 17, no conversion finding) and TC-MOD-28's (`boots.glb` alone on
  PES 21: `model_gltf_unsupported`, folder dropped), re-run green. The "Open for converge" items
  above are ruled at 4.y-conv.

- [x] 4.18 **Hand auto-split (Fox)**: `model_convert::ops::hand_split::split_by_skeleton_group`
  on every face-content FMDL with positive `skh_*_l`/`skh_*_r` weights (never on a boots- or
  gloves-named model or a shared `Boots/`/`Gloves/` folder's), detected by the deep pass, the
  folder given a gloves task, both tasks splitting, `model_hand_split` (I). Plan:
  `model_conversion/hand_split.md` "Pipeline integration"; `team_compiler/pipeline.md` "2.
  Per-export serial steps" step 6 (last paragraph) and "3. Per-model-folder parallel steps" step
  3; `player_folders.md` "At compile time, the pipeline" step 0. IDs: TC-MOD-31. Crates:
  `model_convert` (native detection made public), tc (`deep/`, `validation.rs`, `plan/`,
  `processing/model.rs`, `messages.rs`) → verify: a `/co/` slot 05 folder holding the lead's
  fixture `tests/fixtures/hand_split/body.fmdl` (README): the CPK holds
  `glove/g0625/#Win/glove.fpk` with `glove_l.fmdl` and `glove_r.fmdl` (8 faces each), the face's
  `fcl_hair.fmdl` holds the other 24 of the source's 40 faces, `GloveList.bin` gives player
  71405 glove 625, and `model_hand_split` names `body.fmdl`; the same file as `boots.fmdl` is
  compiled unsplit. Pre-Fox targets split with 4.14, a `.model` source compiled for Fox with 4.17.
  Done 2026-10-07 (Opus 5.5, first time, no lead fix): `model_convert`
  `ops::hand_split::fox_has_hand_weights` (moved from `convert.rs`, now public); `deep/`
  `ContentPass` (`hand_weighted` beside the findings, one parse per FMDL), `ModelRead`;
  `CheckedSource.hand_weighted`; `ExportToPlan` a struct; `plan/mod.rs`
  `ModelFolder.hand_split`, `hand_split_parts` (Fox only), the gloves task and its files;
  `processing/model.rs` `parts_of`, `split_fmdl`; codes `model_hand_split`,
  `model_conversion_failed`; help sentence. Test helper `clean_model()` (a glove written as
  a face model) renames its `skh_` bones so it is not split. A face body left with no face
  is left out (the package then holds `face_diff.bin` alone). Three open issues (Issues,
  hand auto-split). Unseen in game.

- [x] 4.19 **Referees (Fox)**: a `/refs/` export compiled into `refs_cpk_name`'s CPK; slots
  01-35 mapped by `players.txt`, a folder mapped to several slots prepared once and
  instantiated per slot (`referee0NN` face, `k99NN`/`g99NN`; links to a shared `Boots/` or
  `Gloves/` resolve to each linking slot's own); the referees' textures under team 999's common
  folder keyed by the folder name; the Fox referee template tree in the refs CPK; the team CPK
  untouched by referee content. Plan: `blue_port.md` "Referee export processing";
  `pipeline.md` step 6 (the referee layout), texture relocation, "5. Writer" step 5 (the refs
  CPK, its place among the run's CPKs, not the walk's boundary), "Game paths reference"
  referee rows, "Resolved decisions" (embedded templates, the referee trees' override names);
  `player_folders.md` "Multi-mapped processing". IDs: TC-REF-01, 02, 03, 05, 10; TC-OUT-09's
  referee half (`tests/cli/sideload.rs` gains the refs export, `livecpk/` equal to the team
  CPK's and the refs CPK's entries together); TC-DEP-11's refs export. Recon:
  `.tmp/4_19/recon_4_19.md`. Slices:
  4.19-lead done 2026-10-07: `resources/templates/referees_fox/` (Red's `refscpk_fox`, 31
  files, provenance in `resources/templates/README.md`); plan rulings (decision entry "the
  refs CPK among the run's CPKs, and the referee tree's names").
  4.19a done 2026-10-07: referee planning and processing on Fox, output still in the run's
  one CPK (in multi-CPK mode, the teams parts): `paths.rs` `PackageKey { Id, Referee }`,
  `REFEREE_TEAM_ID`; `subset.rs` `referee_not_compiled` (a refs export's kit, logo or
  portrait named), `link_feeds_own_package` (a referee's every link builds his slot's own
  package; shared by the gate, `ids.rs` and planning); `plan/mod.rs` `mapped_folders`,
  generic `player_folders`; help chapter; `tests/cli/referees.rs`. TC-REF-03, 05, 10.
  4.19b done 2026-10-07: the refs CPK: `settings.rs` `refs_cpk_name`; `cli.rs`
  `compile_settings` returns it beside the layout (normal mode), `shared_with_team_side`
  refusals; `compile.rs` sources discovered before the preflight, `cpks`/`promoted` take
  `refs`, `referee_tasks`, only the written CPKs promoted or installed; `writer.rs` `RefsCpk`,
  `Written`, `overridden` (the `admit` split); `bins/installed.rs` the walk passes over the
  refs CPK (`check`'s too); `validation_pass` takes the discovered sources. TC-REF-01, 02,
  TC-DEP-11.
  4.19c done 2026-10-07: the Fox referee template tree: `templates.rs` `REFEREES_FOX` (31
  files, one `include_bytes!` each through `referee_tree!`), overrides from
  `templates/referees_fox/<game path>` (`tree_files`), `Templates::referees_fox`;
  `writer.rs` `Referees` (refs CPK or the sink, sideload) and `finish_referees`: the tree
  after the refs export's entries when it commits; no tree in test mode. TC-OUT-09's
  referee half.
  Moved out: the referee marker (collar 77, `ref_marker.dds`; TC-REF-04, 06, 07, 08) is step
  4.27's; pre-Fox referees are step 4.19d below.

- [x] 4.19d **Referees (pre-Fox)**: `referee0NN.cpk` faces, local boots and gloves as
  `face.xml` entries, `refscpk_prefox` (Red `Engines/templates/refscpk_prefox/`, copied
  lead-first into `resources/templates/referees_prefox/` as 4.19-lead did for Fox), the
  tree chosen by engine in `writer.rs` `finish_referees`. TC-REF-09. Waits on 4.14 (pre-Fox
  export) and 4.20 (the gate withdrawn for pre-Fox). Lead-first done 2026-10-09:
  `resources/templates/referees_prefox/` (Red's `refscpk_prefox`, 51 files, Red at `e12aa01`,
  byte-identical to Blue's and across three Red versions; provenance section in
  `resources/templates/README.md`; `.gitattributes` marks the tree `-text`, three `.mtl` files
  being CRLF, which a plain `git add` had normalized).
  Lead-first (2) 2026-10-09: `resources/templates/collar_empty.model`, FPC's `collar_105.model`
  (852 bytes, one three-vertex mesh, byte-identical to FPC's `referee_collar_105.model`), the
  empty `collar_077.model` the pre-Fox marker pair needs; README section. 4.19d takes 4.27's
  pre-Fox half with it (DECISIONS 2026-10-09 "The pre-Fox marker goes out from the template
  tree's `referee_prop` pair"): since 4.20a a pre-Fox refs export reaches the Fox-only marker
  task, which writes an FMDL at the Fox collar path into the pre-Fox refs CPK. TC-REF-04 too.
  Open after it lands: whether a nocloth `.model` reads the `.mtl` beside it (stock collars
  ship none), an in-game check on PES 17 with the harness (`manual:` line for TC-REF-04).
  Done 2026-10-09 (Opus sidekick, one round, no lead fix): `templates.rs` `REFEREES_PREFOX`
  (51 files, `referee_tree!` taking the folder), `Templates::referee_tree(engine)` replacing
  `referees_fox`, `referee_tree_file`, two override maps (the trees share 21 game paths: the
  configs and `RefereeAppearance.bin`), `collar_empty.model` a `Resource`; `paths::referee_collar`,
  `referee_collar_mtl`, `referee_marker_model(engine)`; `Referees` takes the engine; the marker
  task on pre-Fox writes the texture, the tree's `referee_prop.model` as `referee_collar_077.model`,
  its `.mtl` reduced to `judge_incom` naming the marker texture, and the empty collar. TC-REF-09,
  TC-REF-04, the no-marker and override twins, the `collar_empty.model` override. Four
  contradictions, all accepted: TC-REF-09's wording twice (the tree's `k0062` is a `boots/`
  path; the xml type is `parts`), the pre-Fox prop model is unskinned rather than painted to
  `static` (plan sentence by engine, decision entry), the pre-Fox template configs wear
  collar 26 (the tree's `referee_collar_026`), Fox's 105. The TC-REF-10 probe on PES 17 found
  the shared boots written twice: step 4.19e. Noted for `pes_model`'s converge: the stock
  `referee_collar_026.model` (19-bone group) decodes to 0 positions for its mesh.
  manual: checked 2026-10-09 by the lead on PES 17 through the harness (`.tmp/4_19/ingame/`, results in `.tmp/4_0/apptest/results.txt`): the referee present with the pair installed, nothing drawn; a `.mtl` beside a nocloth `.model` is read and stops the model drawing (runs E and F against 2026-10-08's run I), so the pre-Fox marker goes by Red's route instead: step 4.19f, DECISIONS 2026-10-09; the scene where the pre-Fox marker shows is a maintainer question

- [x] 4.19e **A pre-Fox referee's shared boots or gloves link** (done 2026-10-09, Opus
  sidekick, one round, no rework, lead fix: the help's referee paragraph covers PES 2015 to
  2017; `plan/mod.rs` `blank_face` for a team alone, `TaskKind::files` reads a `.mtl` or a
  pre-Fox model by the package its source feeds; `prefox_face.rs` reads a combined boots or
  gloves source's texture stems alone; TC-REF-13 proven, TC-REF-10 extended; two rulings
  corrected by the sidekick on evidence: the texture stems stay read across sources (the
  face's `.mtl` otherwise kept a Fox directory), and planning's file list still fed the
  shared boots to the face, a probable panic on a shared `boots.skl`): on PES 15-17 a referee's
  link to a shared `Boots/` or `Gloves/` folder is written as his slot's `k99XX`/`g99XX`
  folder alone, as the plan says (`blue_port.md` "a referee's link resolves to his slot's
  `k99XX`/`g99XX`"; `player_folders.md` "A link plus local models combines", pre-Fox: "a
  boots/gloves link keeps loading its shared folder by ID"), and not copied into his
  `face.xml` as well; a referee folder holding only links gets no `referee0NN.cpk` (today it
  gets one holding the shared boots as a `parts` entry beside a dummy `face_neck`, with
  `xml_face_neck_added`: the 4.19d probe, 2026-10-09, `.tmp/4_19/sk_4_19d_report.md` "Probe").
  Settled with it (DECISIONS 2026-10-09): a referee folder with no face model gets no face
  folder on either engine (a team player's gets the blank face, for FPC bodies; a referee has
  none and keeps the game's head), so TC-REF-10's Fox run writes no `referee0NN` face either;
  the converted boots `.mtl` naming `common/999/dummy_kit.dds` is the general reserved-stem
  rule and changes nothing. ID: TC-REF-13 (TC-REF-10 extended). Crates: tc (`processing/prefox_face.rs` or the referee slot instantiation; find
  where the link is folded into the face) → verify: TC-REF-13's run writes `k9901/` and
  `k9920/` and no face CPK.

- [x] 4.19f **The pre-Fox marker by Red's route** (done 2026-10-09, Opus sidekick, one round,
  no rework, lead fix: a test name shortened; `paths::REFEREE_PROP_TEXTURE` and
  `referee_marker(engine)` replace `referee_collar`, `referee_collar_mtl` and
  `referee_marker_model`; the marker task's pre-Fox arm is the converted texture at the
  prop's path alone; `finish_referees` skips the tree's file at the marker's path and the
  configs wear the marker on Fox alone; `collar_empty.model`, its resource and README section,
  `Templates::referee_tree_file` and the prop consts removed; TC-REF-04 rewritten and proven;
  six contradictions accepted, one a fact for the plan: the tree's `incom_bsm.dds` is
  WESYS-compressed where the compiler writes a plain DDS, as Red did): on PES 15-17 the marker task writes the
  converted `ref_marker.dds` as the template tree's
  `common/character1/model/character/parts/referee/incom_bsm.dds` (the texture of the prop the
  game draws by itself), the tree's file left out of the CPK when the task's entry took its
  path (`finish_referees`: a tree file at a path a refs task wrote is skipped, as an override's
  is), no `referee_collar_077` pair, no `collar_077.model`, the configs the template's; the
  `collar_empty.model` resource, its file and README section removed (nothing uses them);
  `paths::referee_collar`, `referee_collar_mtl` and `referee_marker_model`'s pre-Fox arm go
  if no caller stays; `Referees` notes the marker at the engine's marker path (Fox the
  collar, pre-Fox the prop texture) and `wearing_marker` applies on Fox alone. Plan:
  `blue_port.md` "Referee export processing" (the marker bullets, rewritten 2026-10-09 from the
  in-game check: a `.mtl` beside a nocloth `.model` is read and the model then stops drawing,
  runs E and F of `.tmp/4_19/ingame/test_ref04_runs.py`); DECISIONS 2026-10-09 "The pre-Fox
  marker goes by Red's route". ID: TC-REF-04 (rewritten; the 4.19d test's assertions change
  to it) → verify: TC-REF-04's run writes `parts/referee/incom_bsm.dds` as a DDS of the
  marker, no nocloth entry, and the 20 configs byte-equal to the template's; the tree test
  (every tree file at its path) excludes that one path when the marker went in.

- [x] 4.20 **Withdraw the Phase 3 subset gate**: `plan/subset.rs`'s gate (`first_not_compiled`
  and its walk) and `content_not_yet_compiled` removed (the catalog row reads withdrawn),
  TC-OUT-06 withdrawn, every content kind and both engines reach processing; the role helpers
  stay, as `plan/roles.rs`. Plan: `team_compiler/README.md` "Phase 3 scope"; `messages.md`.
  Rulings (lead, 2026-10-09; DECISIONS, eight `team_compiler` entries) from the reconnaissance
  `.tmp/4_20/recon_4_20.md`, which maps each class the gate named to what validation, planning
  and processing do without it (processing already reads only the files a role names, so a
  role-less file is never read; what the gate gave the member was the telling): a file no role
  reads is `file_not_used` (W), a refs export's kits, logo, portraits and collars included; a
  `.model` collar on Fox is `model_conversion_failed` at planning; a shared boots or gloves
  folder with no model takes no id (`shared_folder_no_model`, W); a `Faces/` folder's boots or
  glove model is a part of the linking player's own package on Fox (what combining means); per-kit
  variants where no `face.xml` names the set are left out on pre-Fox as on Fox
  (`kit_variant_model_fox` renamed `kit_variant_model_left_out`); a shared face's `face.xml` is
  ignored on pre-Fox too (`xml_ignored_shared`, I; the question in `QUESTIONS.md`); the Common
  tasks read direct files only, a Common glTF is `model_gltf_unsupported`, a shared folder's
  `.common` link has no role (three lenient-mode panics or wrong outputs); a Fox `.mtl.common`
  link gives a `.model` its set (a panic under default settings once the gate stopped naming the
  link); `kit_texture_not_used` covers a `kit_*` stem outside the seven. IDs: TC-MOD-53 to 57,
  TC-CMN-11 to 13, TC-KIT-30, TC-REF-11, TC-XML-10. Slices: (a) done 2026-10-09 (sidekick):
  the gate and its 33 tests removed, `plan/subset.rs` -> `plan/roles.rs` (`FolderModels::shared`,
  `of_shared`, `admitted`), `file_not_used` in `validation.rs` (`unread_for_a_known_reason`,
  `referee_messages`), a Fox `.mtl.common` link `CommonMaterial` with `common_files` the
  direct Common `.mtl` files on Fox, `CollarFile::NoMaterialSet` -> `model_conversion_failed`
  at planning, `drop_common_gltfs`, the Common tasks direct-only, and, from its report, the
  deep pass's `.mtl` search resolving no shared folder's link (a lenient-mode panic the brief
  missed; `deep::pairings`); TC-MOD-53 to 55, TC-CMN-11 to 13, TC-REF-11. Mutants: 78 (62 caught, 16 unviable: type-driven defaults and an && inside a let chain; the first run's two survivors, the Fox Common .mtl filter's && and admitted, caught by R1's tests).
  (b1) done 2026-10-09: `kit_texture_not_used` for a `kit_*` stem the compiler does not build
  (`drop_unused_kit_textures`), `shared_folder_no_model` (`ids::shared_folders_with_no_model`,
  shared with `shared_folders_taking_ids`; a selected glTF counts as a model), a Common-set
  part's texture stems pointed at the Common output on Fox (`PartTextures::CommonSet`; found at
  a: TC-MOD-54's FMDL kept `./skin.dds`; DECISIONS 2026-10-09), the Common glTF drop by the
  selection order with the beaten other-format model removed and a player whose link names it
  dropped (`selected_common_gltfs`, `linked_common_gltf`; DECISIONS 2026-10-09, sidekick-found:
  the brief's shape panicked), the help's refs and `kit_spec` sentences (TC-KIT-30, TC-MOD-57,
  TC-MOD-54's directory). Mutants: 51 (38 caught, 12 unviable: type-driven defaults; one survivor, the `||` of `linked_common_gltf`'s link filter, caught by a lead input added to the Common glTF planning test, a `.mtl.common` link whose stem a selected glTF holds, which must not drop its player; the kill checked by a hand perturbation). Left for b2: the help passage still
  describing the withdrawn gate (~line 291, "For PES 2018 to 2021 it names a referee export's
  kit"), the help naming `shared_folder_no_model`, a refs kit reported twice (`file_not_used`
  and the map's `kit_texture_not_used`: planning's kit drop runs over a refs export's kits). (b2) done 2026-10-09: per-kit model sets left out on pre-Fox where no `face.xml` names them
  (`roles::leaves_out_kit_variants`, one rule for the roles and the finding; `model_variant_sets`
  by the target's format; `kit_variant_model_fox` renamed `kit_variant_model_left_out`; a lone
  variant under the marker is a part), a shared folder's `face.xml` role-less and
  `xml_ignored_shared` (widened to any shared folder's, sidekick-found), `FolderModels::of_shared`
  taking the folder's kind (ten callers, `roles::shared_folders`), the help's gate passage
  rewritten and `shared_folder_no_model` named, a refs export's kits left to validation
  (TC-MOD-56, TC-XML-10, TC-REF-11 extended). Mutants: 61 (47 caught, 14 unviable: type-driven defaults and an && inside a let chain, 0 missed). 4.20 done: the verify
  criterion re-run at b2 (`content_not_yet_compiled` nowhere in `crates/`; `subset` only in
  `prefox_split.rs`; the acceptance report lists TC-OUT-06 withdrawn). For converge: on pre-Fox any typed
  `.model` gives a shared boots folder a face, so a `face_diff.bin` there takes a face role no
  task reads and is silent. Until 4.19d a pre-Fox
  refs export reaches processing and is written with the Fox referee tree and marker paths
  (`output/writer.rs` `finish_referees`, `referee_marker`): 4.19d chooses the tree by engine and
  is the next step. Crates: tc → verify: `rg content_not_yet_compiled crates/` finds nothing;
  `rg subset crates/tools/team_compiler/src` finds only `prefox_split.rs`'s "subset of its
  materials"; `just acceptance` reports TC-OUT-06 withdrawn and no test citing it.

- [x] 4.21 **Bins from the installed CPKs**: `bins/dpfl.rs` `DpFileList.bin` reader (16-byte
  header, 48-byte records, an all-zero tail of any length ignored; measured on the PES 17 and 21
  files, the PES 15/16/18/19/20 files when an install exists); the walk from
  the entry of next-lower priority than the output CPK upward, each bin from the first CPK that
  holds it (`cpk::CpkArchive`, `wezlib::decompress_if_wrapped`), the supplying CPK reported with
  `bin_source` (I; `bundled` when the embedded base was used); the bundled bases when no install
  or DPFL is found; the `templates/` override directory (`template_override_active`,
  `template_override_unreadable`); FPC patching of kit slots absent from the export from the
  installed `UniformParameter` or pre-Fox kit bins (`kit_config_fpc_adjusted`,
  `kit_config_fpc_unpatched`). Plan: `pipeline.md` "4. Per-export non-model steps" (Bins
  accumulation), "Resolved decisions" (Working-bin lookup, Templates and fallback bins);
  `fpc_toggle.md` "Kit slots absent from the export". IDs: TC-BIN-05..09, TC-BIN-13's CLI
  test, TC-BIN-16, TC-BIN-21. Three slices: (a) the reader, the walk, `bin_source`,
  `dpfilelist_missing`, `installed_bin_unreadable` (05, 08, 09, 13, 21; fixtures in
  `tests/fixtures/dpfl/`, the lead's); (b) the `templates/` override (07); (c) FPC patching
  and a `Full` team's stale kit configs (06, 16). Crates: tc
  (`bins/dpfl.rs`, `bins/mod.rs`, `templates.rs`) → verify: a sandbox install whose DPFL lists
  `4cc_08_bins`, `4cc_61_midcup` and `4cc_99_test` with a `4cc_08_bins.cpk` holding a
  `UniColor.bin` in which team 714's p1 entry is set and a `4cc_61_midcup.cpk` holding one in
  which it differs: compiling `/co/` with only `p2/` leaves p1's bytes equal to the
  higher-priority CPK's and p2's set; with `cpk_name = 4cc_61_midcup` the p1 bytes come from
  `4cc_08_bins.cpk`
  - [x] 4.21a The reader, the walk and its findings, done 2026-10-07 (sidekick, one rework
    round). `bins/dpfl.rs` `entries` (the measured layout; fixtures `tests/fixtures/dpfl/`,
    the lead's, and the repository's PES 17 `examples/DpFileList.bin`); `bins/installed.rs`
    `working_bins` (the CPKs listed below the run's own, nearest first, each bin from the
    first that holds it, unwrapped; stops once every bin is found; a listed CPK with no file
    passed over); `WorkingBins::uniform_parameter`, so `UniformParameter.bin` too builds on
    the installed one; `compile` runs the walk before any export is read, its failure
    `installed_bin_unreadable` (F); `dpfilelist_missing` (E/W, the new
    `CatalogSeverity::ErrorOrWarning`, a Warning with `--no-deploy`), `bin_source` per bin.
    Rework: `studio_core` `CommonSettings::pes_folder` expands `pes_folder_path`'s `**` (the
    walk saw no folder with the default setting). Help paragraph. Every compile's output now
    opens with three (pre-Fox: two) `bin_source` lines; 20-odd tests gained them. TC-BIN-05,
    08, 09, 13, 21. Not yet: `pes_folder_not_found`, `cpk_name_unlisted` (4.24);
    `dpfilelist_cpk_missing` (4.25). Gates green (161 of 253); `clef-diff 43ed77c`: 32
    windows, no flag; `mutants-diff 43ed77c`: 52 mutants, 40 caught, 12 unviable, 0 missed
    (the first run missed 2, an open error other than `NotFound` taken as "no file"; two
    tests added in a second rework, the CPK one a directory on Windows and a self-link on Unix)
  - [x] 4.21b The `templates/` override folder, done 2026-10-07 (sidekick, landed first time).
    `templates.rs` `Templates` (a table of the eight embedded resources by file name, the
    overrides read once by listing the folder so names compare exactly on Windows too;
    `template_override_active` per file, `template_override_unreadable` Fatal before the
    walk); every production use goes through the run's `Templates` (`WorkingBins::bundled`,
    `CompileContext`, `model::package`, the placeholder kit). An override that does not parse
    fails where the embedded one would (a color bin: `cpk_write_failed` at the end; the
    placeholder kit: its kit's task; `body.skl`, `face_diff.bin`, `fcl_hair_sim.fclo` are packed
    unparsed). Help paragraph. TC-BIN-07, and a face package taking `templates/face_diff.bin`.
    Gates green (162 of 253); `clef-diff 68f00cd`: 46 windows, no flag; `mutants-diff
    68f00cd`: 54 mutants, 41 caught, 13 unviable, 0 missed
  - [x] 4.21c FPC patching of absent kit slots, a `Full` team's stale kit configs, bins
    parsed as they are read, done 2026-10-07 (sidekick, one rework round: a mutation
    survivor's test and a move). `bins/kit_configs.rs` `kit_configs` (a `Full` team's configs
    its kits do not name removed, a failed kit's kept; a `Midcup` `fpc_on` team's configs of
    the kits its `UniColor.bin` record holds that the export does not, given the FPC values,
    `kit_config_fpc_adjusted`/`kit_config_fpc_unpatched` on the export naming the slot);
    `UniColorBin::kits`, `kit_slot`; the manifest's `team_kits` (`TeamKits`, every team
    export) replacing `full_team_kits`; `UniformParameter.bin` written whenever the run
    changes it; `WorkingBins` holds parsed bins, the walk and `Templates::read` parsing the
    three bins as they read them (`installed_bin_unreadable`, `template_override_unreadable`).
    The bundled base's 714 configs already carry the FPC values, so a from-scratch compile
    patches nothing. Help sentences. TC-BIN-06, 16, and a CLI test of an installed bin that
    does not parse. Pre-Fox loose configs (TC-BIN-18) stay with 4.14. Gates green (164 of
    253); `clef-diff 366b18c`: 63 windows, no flag; `mutants-diff 366b18c`: 64 mutants, 43
    caught, 21 unviable, 0 missed (the first run missed 1: a `Full` removal with no committed
    config)

- [x] 4.22 **Fox player tables**: `BootsList.bin` and `GloveList.bin` read from the installed CPKs
  by the same walk (the seed rows ride in `4cc_08_bins.cpk`), the (player id, item id) pair of
  every compiled player whose
  boots/gloves output committed replaced with the planned ID, a failed output keeping its row,
  a compiled player the `Full` export gives no boots or no gloves losing that installed row, every
  other row kept, written whole, plain, sorted by id; `PlayerAppearance.bin` read and
  written whole with no row changed (the rows are Phase 5's, with the stock and default-ID
  boots/gloves rows). **External:** the in-game effect of a gloves row waits on FoxDen's gloves
  patch (worklog "Issues"); the table content does not. Plan: `pipeline.md` "Bins accumulation"
  (player appearance tables), "Game paths reference" (the three rows); `settings_toml.md` "Player
  settings in exports" (Fox); `development_plan.md` "Phase 4" `bins/`. IDs: TC-BIN-10..12,
  TC-BIN-17, TC-BIN-22.
  Crates: tc (`bins/mod.rs`, `paths.rs`) → verify: PES 21 compile of `/co/` with slot 05's
  `boots.fmdl` over an installed `BootsList.bin` of ten pairs: the CPK's table holds eleven pairs
  sorted by id with (71405, 625) among them and the ten unchanged; `GloveList.bin` is
  byte-identical to the installed one; `PlayerAppearance.bin` is byte-identical to the installed
  one. Done 2026-10-07 (sidekick, landed first time; lead fix: the parity test, the lead's,
  now expects exactly the tracer's two `player_table_missing` warnings, the sidekick's
  contradiction, accepted). `bins/player_tables.rs` (`ItemTable`, `ItemList`: little-endian
  u32 pairs, measured on the game's and the cup's tables; `read_player_appearance`, a 60-byte
  row check); the walk's three Fox-only tables, no `bin_source` for one not found;
  `plan/item_rows.rs` (`ItemRow`, `RowChange`, `export_rows`: each compiled player's row from
  the `Models` task building his boots or gloves, his own folder's or a linked shared one's;
  `Remove` for a `Full` export's player with none); the writer records committed batches and
  `add_player_tables` writes each found table whole after `UniColor.bin`; a table not found
  is not written, `player_table_missing` (W, new) counting the committed rows left out. Help
  paragraphs. Eight existing tests gained the warning (Fox, boots or gloves, no install).
  Gates green (169 of 254); `clef-diff 53c750e`: 44 windows, no flag; `mutants-diff
  53c750e`: 56 mutants, 43 caught, 13 unviable, 0 missed

- [x] 4.23 **Output sink and modes**: `output/sink.rs` `OutputSink` (CPK, loose folder) fed by
  output-relative paths; `processing/materialize.rs` the one seam (relocation and FPK packing,
  skipped in test mode); `--mode test` writing `output/test_output/<canonical source key>/` with
  export-relative processed entries and no FPK; `--mode sideload` replacing the whole contents of
  `{pes_folder_path}/livecpk/` with the run's game-path tree, refused on PES 15/16 (exit 2);
  artifact routing per mode (`teamnotes.txt` under `output_folder_path` in every mode; sideload:
  bins, overrides and referee content at their game paths in `livecpk/`; test: bins under
  `test_output/_bins/` at game-relative paths, referee content per export like a team's,
  overrides not applied). **External:**
  FoxDen's LiveCPK gaps (worklog "Issues": 2019/2021 sites, PES 2020, path length, in-match
  loads) decide the in-game effect only; the written tree is what the scenarios test. Plan:
  `pipeline.md` "5. Writer" step 5 and the output-modes paragraphs, "Resolved decisions"
  (Output-mode artifact routing); `settings.md` "CLI", "Path resolution"; decision entry
  "sideloading through FoxDen (4.0e)". IDs: TC-OUT-07..11, TC-OUT-17. Crates: tc (`output/sink.rs`,
  `processing/materialize.rs`, `cli.rs`) → verify: `--mode test` on the tracer writes
  `output/test_output/egg Midcup Tracer/Players/05 - The Chad Stormworks Player/fcl_hair.fmdl`
  with its texture path rewritten and no `.fpk` anywhere under `test_output/`; `--mode sideload`
  with a stale `livecpk/old.txt` removes it and writes files whose relative paths and bytes equal
  the entries of a normal-mode CPK of the same export
  - [x] 4.23a the output sink and sideload mode (TC-OUT-09 but its referee half, which lands
    with 4.19; TC-OUT-10): done 2026-10-07 (Opus 5.5, first time, no lead code fix).
    `output/sink.rs` `OutputSink` (`Cpk`, `Loose`, the loose one refusing a file twice),
    `CpkOutput` writing through it; `compile::OutputMode` (`Normal { no_deploy }`, `Sideload {
    pes_folder }`) from `cli::output_mode`, refused on PES 15/16 and without a PES folder;
    `deploy::promote_livecpk` (rename, copy when the rename fails). Every sideload test points
    its settings at the sandbox (the default `pes_folder_path` is the real install). Gates green
    (174 of 254); Clef 37 windows, no flag; `mutants-diff 202e040`: 35, 21 caught, 13
    unviable, 1 missed (the `NotFound` guard in `promote_livecpk`, missed on the Linux half),
    killed by the lead's test of a previous tree held open, Windows-only: no portable way makes
    the removal fail while the copy succeeds (first written "the Linux half runs as root"; it
    runs as `debian`, `systemd-run --uid=debian`, found at 4.y-fix2), so a Linux half may
    report it again
  - [x] 4.23b test mode and the materialize seam (TC-OUT-07, 11, 17): done 2026-10-07 (Opus
    5.5, first time, no lead code fix). `processing/materialize.rs` (`TaskOutput`,
    `EntryTarget` in `CompileContext.target`, `materialize` the one place entries get their
    paths; the `.fpk`/`.fpkd` packing moved there from `model.rs`, whose `package` returns the
    package's files); `OutputMode::Test` (loose sink at `<staging>/test_output`, no overrides
    listed, the writer's `bins_prefix` `_bins/`); `deploy::promote_tree` shared with sideload.
    Gates green (177 of 254); Clef 58 windows, no flag; `mutants-diff 93ba74c`: 32, 24 caught,
    8 unviable, 0 missed. The parity test keeps reading the CPK (decision entry)

- [x] 4.24 **Deployment**: each staged CPK copied to `download/{name}.cpk.partial` and renamed over
  the old one, the staging folder removed; the preflight before any export is
  read (`download/` probe; `pes_version_mismatch` W); degradation to `output/` with
  `pes_folder_not_found`, `dpfilelist_missing`, `cpk_name_unlisted` (against the installed list
  alone until 4.25 splits off `dpfilelist_outdated`, a name the bundled list has),
  `old_cpk_locked` (the rename failing), `deploy_target_unwritable` (the probe or the copy
  failing; `elevation::is_access_denied` at the probe); `--no-deploy`
  unchanged; stale `.staging/` folders of dead runs removed at start (the run's lock file); the
  test harness points every run at the sandbox's PES folder (lead, first); deployment adds no exit
  code (a degraded run exits 1, an aborted one 3, as the mapping already says). Plan:
  `pipeline.md` "6. Post-processing" (Staging, Deploy CPKs, Degraded run, `--no-deploy`,
  Destination writability preflight); `messages.md` "Output stage and savefile"; `settings.md`
  "CLI" (exit codes). IDs: TC-DEP-01..07. Crates: tc (`output/deploy.rs`, `cli.rs`) → verify: a
  sandbox PES folder with `PES2021.exe`, a DPFL listing `4cc_99_test` and an old
  `download/4cc_99_test.cpk`: `compile` leaves `download/4cc_99_test.cpk` equal to the staged
  bytes, no `.partial`, nothing else in `download/`, nothing in `output/`, exit 0; with the old
  CPK held open by the test: `old_cpk_locked`, `output/4cc_99_test.cpk` holds the run's CPK, the
  old one is byte-identical, exit 1. Done 2026-10-07 (Opus 5.5, first time; lead fixes: the
  sweep after the run's own lock, a Linux-only test failure, two survivors' tests).
  `deploy::preflight` (before `working_bins`), `deploy::deploy` (`DeployFailure::{Copy,
  Rename}`), `Staging` (lock file, `Drop` removing the folder, the lock and an emptied
  `.staging/`; `sweep`), `probe_folder` shared with `prepare_output_folder`; the five codes;
  152 test call sites given `--no-deploy` by script; the GUI's Compile deploys too (its tests
  install a sandbox PES). Gates green (184 of 254); Clef 44 windows, no flag;
  `mutants-diff 6d51094`: first run's Linux baseline failed (a `face/` substring check met the
  sandbox path in the new `deploy_skipped_by_flag` line; fixed), rerun 41, 28 caught, 9
  unviable, 4 missed on the Linux half, all the `download/` probe's (TC-DEP-04 is Windows-only):
  three killed by a portable test of a missing `download/`, verified by hand; the
  access-denied guard is caught only on Windows (root ignores permissions), so a Linux half
  may report it again

- [x] 4.25a **The default `cpk_name` is `4cc_99_test`** (lead; mechanical): `4cc_90_test`
  renamed in the code and its tests (81 mentions in 17 files of `crates/`), the plan having
  it already. Done 2026-10-03, by script; gates green. The tracer fixture's README keeps
  `4cc_90_tracer`, the name Red's golden run used → verify: no `4cc_90_test` left under
  `crates/`, `just gates` green. Re-read at converge (2026-10-09): `4cc_90_test` is the old
  DLC's real CPK name and has since come back in `upgrade.rs`'s rename table, `dpfl.rs`'s
  real lists, their fixture README and a test (4.21a, 4.25c); the criterion means no
  `4cc_90_test` as the default or a test run's CPK name, which holds (`settings.rs`
  defaults to `4cc_99_test`, pinned by tests).

- [x] 4.25 **DpFileList upgrade and the official-list check** (the entry list is fixed:
  `resources/templates/DpFileList.txt`, 53 entries; before this step, 4.25a renames the
  default `cpk_name` in the code): the one official `DpFileList.bin` embedded; on every
  compile with a PES folder the installed list's entries compared with it in order,
  `dpfilelist_not_official` (W) for any difference that still lists the run's targets, the
  run compiling and deploying all the same (TC-DEP-12) (lead first: the
  file written from the maintainer's entry list in the layout measured at 4.21, as a
  lead-authored fixture with a provenance README; no installed list has the slot run, PES 17's
  still lists `4cc_40_faces` and `4cc_45_uniform`; the brief confirms with the reader of 4.21
  that one file serves every PES version, and reports if a version needs its own bytes);
  `upgrade-dpfl [--yes]`: the entries the official list lacks printed with the size of each
  matching `download/*.cpk`, nothing written without `--yes`, the installed file replaced byte
  for byte with `DpFileList.bin.bak` kept, an old DLC's CPKs renamed by stem to the official
  names (TC-DEP-13), the empty placeholder CPK written for every official entry with no file
  in `download/` (an existing file is never overwritten), no CPK ever deleted by the command;
  `dpfilelist_cpk_missing` (W) at every compile for a listed CPK with no file (TC-DEP-14);
  `dpfilelist_outdated` names the subcommand, and takes from 4.24's `cpk_name_unlisted` (judged
  on the installed list alone) the names the bundled list holds. Plan: `pipeline.md` "6. Post-processing"
  (DpFileList upgrade); `settings.md` "CLI" (`upgrade-dpfl`). IDs: TC-DEP-09, 10, 12..14
  (TC-DEP-08 needs multi-CPK mode: 4.26; here `dpfilelist_outdated` is tested on a single-CPK
  run). Lead first, done: `resources/templates/DpFileList.bin` and `placeholder.cpk` (decision
  entry "the placeholder CPK is the shipped file, embedded"). Slices: 4.25b the official list
  as a `templates/` resource and the preflight's comparison (TC-DEP-12, 14, `dpfilelist_outdated`;
  decision entry "the official-list check: only a compile that deploys, and its findings'
  context"), 4.25c `upgrade-dpfl` (TC-DEP-09, 10, 13). Crates: tc (`templates.rs`,
  `output/deploy.rs`, `cli.rs`) → verify: an installed DPFL lacking `4cc_41_teams` beside a
  1 KiB `download/4cc_40_faces.cpk`: `upgrade-dpfl` prints `4cc_40_faces` as renamed to
  `4cc_41_teams` and exits without writing; `--yes` makes `DpFileList.bin` equal to the
  embedded list and `DpFileList.bin.bak` equal to the old file, `4cc_41_teams.cpk` holding
  the old file's bytes. 4.25b done 2026-10-07 (Opus 5.5, first time; TC-DEP-12, 14 proven;
  mutants-diff 32: 27 caught, 5 unviable, 0 missed; Clef 24 windows, no flag; lead fix: a
  doc comment rewrapped). 4.25c done 2026-10-07 (Opus 5.5, first time; TC-DEP-09, 10, 13
  proven; every expected value of the brief's PES 2017 case matched; mutants-diff 84: 76
  caught, 5 unviable, 3 missed, each killed by a lead test checked against its mutant by hand:
  an official list with an empty slot, an unofficial list needing no rename or placeholder, a
  list that cannot be read; Clef 28 windows, 1 flag rejected). The list replaced last is
  tested on Windows only (a held-open CPK): Linux renames over an open file.

- [x] 4.26 **Multi-CPK mode**: `multicpk_mode` honored (the Phase 3 refusal removed); slots from
  the DPFL entries matching `{prefix}_{NN}_{teams_cpk_name}` exactly, ordered by number; whole
  teams placed first-fit by exact size under `cpk_part_max_size`; every unfilled slot written as
  the empty placeholder CPK, the shipped one embedded (`resources/templates/placeholder.cpk`,
  4.25);
  `cpk_slots_exhausted`, `cpk_team_exceeds_cap`, `cpk_size_over_limit` (single-CPK); the bins CPK
  (`bins_cpk_name`), the
  refs CPK beside them (with 4.19); deployment per generated CPK, all or none
  (`dds_compression = auto` moved to the pre-Fox steps: decision entry "multi-CPK mode: the
  official list's slots, permits given back, all-or-nothing install"). Slices: 4.26a the `cpk`
  crate tells a writer's size were entries added (the cap is checked TOC included); 4.26b the
  parts, placeholders, bins CPK and the three size findings, with `--no-deploy` (TC-OUT-12..16;
  a deploying multi-CPK run still refused); 4.26c deploying them all or none and each judged
  by the preflight (TC-DEP-08, TC-DEP-11 without its refs export, which joins at 4.19).
  4.26a done 2026-10-07 (Opus 5.5, first time): `CpkWriter::len_with`, the finished length
  with more entries added, from the TOC/ETOC layout `finish` shares (`tables`); the added
  entries carry no modification time, as the Team compiler's, since an entry without one
  drops the ETOC (the sidekick's contradiction, accepted); mutants-diff 32: 30 caught, 2
  unviable; Clef 1 flag rejected.
  4.26b done 2026-10-07 (Opus 5.5): `CpkLayout` (`Single` or `Parts`, decided by
  `cli.rs` `compile_settings`: `Parts` only for a normal `--no-deploy` run, a deploying one
  refused with exit 2); `output/parts.rs` `slots` (the official list's exact-stem entries by
  number) and `TeamsParts` (a team's entries held until its export's last task is decided,
  then placed first-fit with `len_with`, `cpk_team_exceeds_cap` and `cpk_slots_exhausted`
  as `Unplaced`, placeholders written for the slots left); the writer releases a held
  batch's permit when held; `cpk_size_over_limit` on a single CPK; settings
  `teams_cpk_name`, `cpk_part_max_size`, `bins_cpk_name`; `size_text` moved to
  `messages.rs`; `check` looks below `bins_cpk_name` in multi-CPK mode. TC-OUT-12..16
  proven. Gates green (lib 484, cli 235); mutants-diff 86: 68 caught, 17 unviable, 1
  survivor (`Unplaced`'s `Display`), its test added by the lead; Clef 63 windows, no flag.
  Left for 4.26c: no check that a path arrives once across parts (each part's writer
  refuses a duplicate within it).
  4.26c done 2026-10-07 (Opus 5.5): a deploying multi-CPK compile; `deploy::preflight`
  judges every CPK of the run (`CpkLayout::cpks`, the order defined once): one
  `dpfilelist_outdated` and one `cpk_name_unlisted` joining their names, `cpks_missing`
  leaving out every CPK of the run, the probe per CPK; `deploy::deploy` copies every
  `.partial`, then one CPK renames over its old one as before and several go through
  `install_all` (old CPKs moved aside to `.cpk.old`, `.partial`s renamed in, `.old`s
  removed; `undo` on a failed rename), `DeployFailure::Rename` naming its CPK; a failure
  promotes every CPK; `TeamsParts` refuses a path placed twice across parts. Lead fix:
  `compile_settings` refuses a `teams_cpk_name` that is the bins CPK's own stem (decision
  entry). TC-DEP-08, TC-DEP-11 (without refs) proven. Gates green (lib 492, cli 239);
  mutants-diff 41: 29 caught, 12 unviable, no survivor; Clef 46 windows, no flag.
  Plan: `pipeline.md` "5. Writer" step 6 ("Multi-CPK mode: teams parts"); `settings.md`
  (`multicpk_mode`, `teams_cpk_name`, `cpk_part_max_size`, `bins_cpk_name`). IDs: TC-OUT-12..16,
  TC-DEP-08 (moved from 4.25: it needs this mode), TC-DEP-11. Crates: tc (`output/writer.rs`,
  `output/deploy.rs`, `settings.rs`) → verify: `/co/`, `/a/` and `/b/` exports with
  `cpk_part_max_size` set just above the first two teams' compiled size and a DPFL reserving
  `4cc_41_teams`..`4cc_45_teams`: `4cc_41_teams.cpk` holds the first two teams whole,
  `4cc_42_teams.cpk` the third, `4cc_43`..`45` are each 6,272 bytes equal to the placeholder
  fixture, `4cc_08_bins.cpk` holds the bins and nothing else

- [x] 4.27 **Referee marker as reserved collar 77, both engines** (lead first: the two
  marker models bundled as referee templates with a provenance README, from the sources
  named under "Phase 4 open questions", each checked to be painted to `static`): the marker model bundled as a
  referee template and emitted in the refs CPK as collar 77, its texture path naming
  `ref_marker.dds` converted into the referees' Common output, the referee template kit configs
  naming collar 77; on a regular team, `collar_id_conflict` for `Collars/collar_77.*` and
  `kit_collar_reserved` (kit dropped) for a kit whose effective collar or winter collar is 77
  (the kit half lands with 4.9 if that step comes first); nothing written outside the refs CPK
  (no `dt00_x64.cpk` write, no setting). Plan: `blue_port.md` "Referee export processing";
  `messages.md` "Referees", `collar_id_conflict`; `pipeline.md` "Collars". IDs: TC-REF-04,
  TC-REF-06..08. Crates: tc (`processing/referee.rs`, `processing/team_assets.rs`,
  `processing/kit.rs`), resources → verify: a refs export with `ref_marker.dds` compiled for
  PES 21 beside a sandbox `dt00_x64.cpk`: the refs CPK holds `nocloth/#Win/collar_077.fmdl`
  whose texture path resolves to the marker's FTEX under `common/999/sourceimages/`, each
  referee kit config decodes with collar 77, and `dt00_x64.cpk` is byte-identical; a `/co/`
  export with `p1/config.toml` naming collar 77 reports `kit_collar_reserved` and p1 is absent
  from the CPK
  4.27-lead done 2026-10-07: `resources/templates/referee_marker.fmdl` (the refs compiler's
  `referee_prop.fmdl`, every vertex weighted to `static` only, read with the `fmdl` crate) and
  its README section; plan: the marker's texture pointed at `common/999/sourceimages/
  ref_marker.dds`, a failed marker counts as absent, replaced configs get collar 77 too
  (decision entry). Slices:
  - [x] 4.27a the Fox half: done 2026-10-07. `TaskKind::RefereeMarker`
    (`processing/referee_marker.rs`: the marker through `texture::common_texture`, shared
    with `common_textures`, and the bundled model repointed with `rewrite_texture_paths`,
    both or a failure); `paths::REFEREE_MARKER_COLLAR`, `collar(id)` (Fox),
    `REFEREE_KIT_CONFIGS`; `writer.rs` `Referees.marker` (noted at the collar's path) and
    `wearing_marker` in `finish_referees`; `kit.rs` `reserved_collar_field` after FPC
    (`kit_collar_reserved`, context `field`, the kit being the scope). TC-REF-06, TC-REF-08,
    TC-REF-07's kit half (the acceptance report counts TC-REF-07 proven; its
    `collar_id_conflict` half is 4.9's). Left as found, none reached by a real export: the
    deep pass does not check `ref_marker.dds` (only its conversion fails it); a
    `Common/ref_marker.dds` beside it gives both tasks one path; an `overrides/` file at
    `collar_077.fmdl` leaves the configs at 105.
  - [x] 4.27 pre-Fox half (folded into 4.19d on 2026-10-09, landed with it): the marker as
    `referee_collar_077.model` with its `.mtl` beside an empty `collar_077.model` (the pair the
    referee needs, in-game 2026-10-08: decision entry; `blue_port.md` "Referee export
    processing"; TC-REF-04). The marker model is the pre-Fox template tree's `referee_prop`
    pair, the empty collar the bundled `collar_empty.model` (lead-first 2026-10-09).
  Moved out: TC-REF-07's `collar_id_conflict` half (landed with 4.9b1, which reserves 77
  beside 105); the pre-Fox marker (TC-REF-04) needs pre-Fox referees (4.19d).
  Settled 2026-10-07 (maintainer): the game needs the loose referee configs but reads their
  values from the `UniformParameter.bin` entries, as for team kits; so 4.27b.
  - [x] 4.27b **Fox referee configs as `UniformParameter.bin` entries too** (done 2026-10-09,
    Opus sidekick, one rework round, the multi-CPK refs-only test the sidekick itself flagged as uncovered; no lead fix: `writer.rs` `finish_referees` pushes each Fox
    config written, the override check first, to `uniform_parameters`; `finish_team`'s early
    return stays out when those are non-empty, so a refs-only Fox run writes the team side
    (TC-REF-01 now both CPKs); TC-REF-12, TC-REF-08 extended, a `templates/referees_fox/`
    config replacement as its entry, the help; one contradiction accepted: the bases hold
    no `referee_1..5` entries, plan and decision entry corrected): on Fox, each
    referee kit config the refs CPK holds loose (the templates' or `templates/referees_fox/`'s,
    with collar 77 when the marker is emitted) also replaces the entry of its name in the
    bins CPK's `UniformParameter.bin`. Plan: `blue_port.md` "Referee export processing" (the
    paragraph on the template kit configs); `pipeline.md` "Bins accumulation". Crates: tc
    (`writer.rs` `finish_referees`, the bins writer). ID: TC-REF-12 → verify: a refs export
    with `ref_marker.dds` compiled for PES 21 alone: the team CPK is written, and its
    `UniformParameter.bin` entries named as the 20 loose configs (`referee_ACL_1.bin` to
    `referee_SDA_3.bin`; the bases hold ACL and DEF, PES 18's CL too, every one at collar
    105) decode with collar 77 and equal the loose files byte for byte, the other entries
    the base's; without `ref_marker.dds` they equal the templates' configs; for PES 17 no
    team CPK. The bases hold no `referee_1..5` entries, as the brief assumed from a byte grep
    (those strings are kit texture names inside the configs; the sidekick's parser,
    `.tmp/4_27/sk_probe/uniparam_names.py`). Brief `.tmp/4_27/brief_4_27b.md`.

- [x] 4.28 **Complete memory accounting**: a running task charges the budget for what it
  allocates (decoded textures, parsed and merged models, its packed entries), without waiting;
  a multi-CPK run's held team stays outside the budget (`pipeline.md` "Writer") and the
  conversion cache is charged with its eviction (4.y). Plan: `pipeline.md` "Admission" (the
  paragraph "Charges while a task runs"), "Resolved decisions and open questions" (Complete
  memory accounting); `libs/pipeline.md` "Memory budget"; decision entry "memory
  accounting: a running task charges without waiting". IDs: none (not user-observable; the
  measurement is the proof). Crates: pipeline (`MemoryBudget::charge`, `peak`), tc → verify:
  a textures task on a texture of known dimensions, run alone, peaks at its source charge
  plus the RGBA size of every level plus the file's size; a batch's output permit equals its
  entries' bytes; a compile with a cap below one decoded texture completes. Done
  2026-10-07 (Opus 5.5, first time; one lead test fix): `pipeline` `MemoryBudget::charge`
  and `peak` (`BudgetState::take`, shared with `acquire`); `CompileContext.budget`, the
  coordinator's (its separate `budget` parameter removed); `texture::decode_charge` (every
  level's RGBA from `probe`, plus the source's size) at `texture::convert` (a cache hit
  charged all the same), `portrait`, the kit's `relaid_main_texture` (and its re-laid
  copy), `derived_colors`, `kit_layout::with_kept_blocks`, the logo's decodes (held in
  `LogoSource`); `model::package` charges its FMDL parts' size; `TaskBatch.output`
  (`output_len`: entries and the kit config) charged in the spawn closure, released in
  `commit` with the permit. Lead fix: the existing held-permit test went through `submit`,
  which drops every batch it decides, so it passed with the release removed; it now calls
  `commit` (red shown). Not measured by a test: the charges in `portrait`,
  `derived_colors`, `relaid_main_texture`, `with_kept_blocks` and the logo.

- [x] 4.29 **A texture a model names must exist** (after 4.21, whose walk of the installed CPKs
  it reuses): `fmdl_texture_not_found` on Fox for a mesh's texture supplied by nobody: stem not
  in the folder, path naming the team's Common output, and neither the export's `Common/` nor an
  installed CPK's table of contents holding it, the CPKs searched being those the installed
  list names before the CPK being compiled, nearest first, never a later one; an Error dropping
  the folder, a Warning keeping it when the lookup cannot be made; a texture `.common` link with no target in the export
  satisfied by an installed CPK the same way; paths naming anything else kept and not looked
  up. The pre-Fox half (`mtl_texture_not_found`) lands with 4.15 under the same rule. Plan:
  `pipeline.md` "Resolved decisions" ("A texture a model names must exist"); `messages.md`
  (`fmdl_texture_not_found`, `mtl_texture_not_found`). IDs: TC-TEX-05, TC-TEX-11, TC-CMN-06 (the
  `dummy_kit*` stems the checks skip; moved here from 4.11). Crates: tc (`check.rs`,
  `processing/model.rs`, `bins/`) → verify: TC-TEX-05's three runs (the CPK holding the
  texture, lacking it, no PES folder): no finding, Error with the folder out of the CPK, Warning
  with the folder in it
  - [x] 4.29a the model check (TC-TEX-05, TC-CMN-06): done 2026-10-07 (Opus 5.5, first time, one
    lead fix: a doc line rewrapped). The working-bin walk opens every CPK listed before the run's
    and keeps their entry paths, folded (`InstalledPaths`, `Unknown` when the walk cannot be
    made); `fmdl::ops::paths::used_texture_paths`; `processing/model.rs` `texture_supply` after
    each part's paths are pointed; `fmdl_texture_not_found` as a task failure (Error, package
    left out) or a `Keep` finding per texture (Warning), the catalog's `ErrorUnlessKept`.
    Contradiction accepted: shared boots and gloves folders now carry the export's Common
    stems (`plan/mod.rs`), else a shared model naming a `Common/` texture was a false Error.
    Gates green (171 of 254); Clef 55 windows, no flag; `mutants-diff 7f1fc58`: 35, 24
    caught, 11 unviable, 0 missed (remote peak 9.00 GiB, at the cap, no build killed)
  - [x] 4.29b the texture link an installed CPK satisfies (TC-TEX-11): done 2026-10-07 (Opus
    5.5, first time, no lead code fix). `aesthetics_export`'s `ValidationContext` carries
    `installed_common_textures` (decision entry; `ResolvedLink::common_target_missing` decides
    both the finding and the link's removal); the tool reads the team's ID from the parsed
    export before validation (`validation.rs` `installed_common_textures`, empty on pre-Fox);
    `walk` takes a `take` closure, so `check`'s `installed_paths` reads tables of contents only
    and reports nothing. Contradiction accepted: the test model names `hair` outside the Common
    output, so only the kept link points it there. Gates green (172 of 254); Clef 33 windows,
    no flag; `mutants-diff 3b332e7`: 39, 26 caught, 13 unviable, 0 missed. Step 4.29 is done

- [x] 4.30 **Marker names**: done 2026-10-03 (Opus 5.5, first time; one contradiction accepted:
  it also renamed the markers in `pes_savefile`'s injected `settings.toml` comments, outside its
  crates). The FPC markers are `fpc_on`/`fpc_off`, a bare `fpc` reads as on, and a kit's menu
  icon is the empty marker `icon_<N>` (`Marker::Icon`; 0-23, zero padding optional; two
  markers, or a number above 23, are `kit_icon_invalid` and the default icon applies), so
  `icon.txt` left the small metadata the readers load; every marker tolerates `.txt`; the old
  spellings are plain disallowed files. Renamed in every plan section, the glossary and the
  fixtures (the tracer's `Kits/g1/icon_11`, the three source archives regenerated by the
  lead). TC-STR-12 and TC-KIT-08 reworded. Files: ae `conventions/file_types.rs`,
  `conventions/mod.rs`, `listing.rs`, `validate/{kits,folders,mod,tests}.rs`; ps
  `settings_toml/{keys,tests}.rs`; tc `plan/subset.rs`, `reader/source.rs`,
  `tests/cli/compile.rs`. Gates green (99 of 203); `mutants-diff a89f0b6`: 41, 35 caught, 6
  unviable, 0 missed. `rg "fpc\.on|fpc\.off|icon\.txt" crates` leaves only the tests that
  assert the old names are refused. Left for Phase 5: `keys.rs`' boots/gloves comments still
  say "no marker: untouched", where `settings_toml.md` has said "otherwise 0" since step 4.0

- [x] 4.31 **Pre-Fox parity reference** (lead, before 4.14): a small pre-Fox export cut from one
  in the maintainer's library (one player with face, boots and gloves, one kit, as the Fox
  tracer was cut), its Studio-layout twin migrated by hand, and Red's CPK for the old one
  compiled for PES 17 with the tracer's settings (`tests/fixtures/tracer/README.md` "`red/`":
  no PES folder, so every bin is built on Red's fallback base), committed under
  `tests/fixtures/tracer_prefox/` with a provenance README if under 1 MB, as a hash manifest
  otherwise. Without it every pre-Fox byte of 4.14-4.17 is unchecked until Phase 6 → verify:
  the README's command reproduces `red/` byte for byte from `old/`
  Done 2026-10-07 (lead, from the maintainer's library as they suggested):
  `tests/fixtures/tracer_prefox/` (/jp/'s Summer 18 export, player Fumos with face, boots and
  gloves in his `face.xml`, kit g1; large textures block-reduced; README), scripts in
  `scripts/provenance/tracer_prefox/`, `tests/parity_prefox.rs` (outer rows, a row per entry of
  the nested face CPK, the `face.xml` table), the sandbox teams list's `/jp/` 731. The twin
  holds no kit until 4.16 (a pre-Fox kit refuses the export), so the kit's rows are "not
  produced" and `UniColor.bin` is deferred. Found: models byte-identical; Red keeps a team
  player's textures in his face CPK (relocated by us as on Fox, same pixels); our DDS header
  is canonical and WESYS unwrapped; `face.mtl`'s missing texture stays `./`; Red keeps
  absent number-texture names in the kit config (compare at 4.16). `red/` is what the README's
  command produced from `old/` in one run; a second run showing it reproduces byte for byte
  was not made. Checks shown to fail: a model's name, the encoded codec, a `TeamColor.bin`
  byte.

- [x] 4.32 **Number atlases re-arranged across engines** (lead first: the measurement, done
  2026-10-07: `scripts/provenance/kit_uv/number_atlas/`, the plan's slot rule and decision
  entry "number atlases: a digit moves by a uniform scale into Konami's slots"):
  neither engine reads the other's `_back`, `_chest` and `_leg` arrangement (ten digits in
  a column on PES 15 to 17, 128×2048 or 64×1024; in a row on PES 18 to 21, 2048×256 or
  1024×128), so an atlas in the other engine's arrangement is re-arranged for the target,
  told by its shape alone, `_name` untouched. The cells are not whole pixels (2048 over
  ten), and a column cell and a row cell differ in proportion, so the lead first measures
  where each game takes each digit from (the stock atlases and how the number meshes map
  them) and records it as a table with a provenance script, as `KIT_LAYOUT_REMAP` was;
  then the re-arrangement is briefed. Plan: `pipeline.md` "4. Per-export non-model steps"
  (the glyph atlases). IDs: TC-KIT-26. Crates: tc (`processing/kit.rs`,
  `processing/kit_layout.rs`) →
  verify: a column atlas whose ten cells are ten flat colors, compiled for PES 21, comes
  out as a row atlas with the ten colors in digit order, and the reverse for PES 17; an
  atlas already in the target's arrangement is byte-identical to today's output. Done
  2026-10-07 (Opus 5.5, first time; one lead test from a mutation survivor, a glyph that
  rounds to no texel on a 2x6 column): `kit_layout.rs` `NUMBER_ATLAS_SLOTS`,
  `atlas_arrangement` (the shape), `number_atlas_rearranged` (one `GlyphMove` per digit,
  `write_glyph`, the top-left texel as filler, one level, `authored_mips` false),
  `rounded_ratio` (integer half-up rounding, now shared with `scaled`); `kit.rs`
  `number_atlas` for `kit_back`, `kit_chest`, `kit_leg` whatever the marker. Contradiction
  accepted: the shape is read with `probe` before any decode, so an atlas left as it is
  is decoded once, by its conversion, as before. The PES 21 half of TC-KIT-26 is tested
  from the CLI uncited; its PES 17 half moved to 4.16. Unseen in game.

- [x] 4.33 **`name.y` is PES 21's value on every version**, done 2026-10-07 (sidekick,
  landed first time, no lead fix): `kit_config` `binary.rs` decodes and encodes Name Y with
  one 6-bit formula (0x1D bit 0, 0x1C bits 3-7) and the 0x1C remainder mask 0x07 on every
  version; `field_limits` caps it at 33 below PES 21 (39 on 21); `UNKNOWN_MASKS` 0x1C is
  0x07, so a TOML `[unknown]` `"0x1C"` of 8 to 15 is refused like any known bit; the TOML
  comment reads `0-39 (PES 15-20: 0-33)`. The private `binary::decode` no longer reads its
  version (`_version`); `KitConfig::decode` keeps the parameter, symmetric with `encode`,
  its doc saying the bytes decode alike. Tests: the template round-trips on PES 17 and 21
  (not 15: its pattern remap rewrites the template's pattern 13 as 11, existing behavior);
  a PES 17 config with 0x1C = `FA` reads Name Y 31, remainder 2, and round-trips; packing
  is identical on 15, 17, 20, 21; TC-KIT-17 (36 → 33 on PES 18), TC-KIT-24 (30 gives
  `F2 0E` on PES 18 and 21, no finding); `deep/documents.rs` and the help paragraph follow.
  `python_bindings` does not use the crate. Gates green (149 of 250); `mutants-diff
  53abc6f`: 24 mutants, 22 caught, 2 unviable, 0 missed; `clef-diff 53abc6f`: one flag, the
  window of the `legacy.rs` comment fix, rejected

- [x] 4.34 **Coverage tag: `Full` or `Midcup` in a team export's name** (4.34a and b done; the rest with the steps listed last) (decision entry of
  2026-10-04; before 4.21, whose TC-BIN-05 and 06 are Midcup cases). Plan:
  `aesthetics_export/object_model.md` "Validation semantics" (Coverage tag, `ExportCoverage`,
  the two `coverage` fields); `team_compiler/pipeline.md` "Bins accumulation";
  `messages.md` `export_tag_missing`. Slices:
  - [x] 4.34a the tag is read and required, done 2026-10-07 (sidekick, landed first time, no
    lead fix): `aesthetics_export` `parse/identity.rs` `ExportCoverage` and `coverage` (the
    second token of `team_name`'s split, now a shared `tokens`; `Full`/`Midcup` in any
    case), the draft's `coverage` (`Full` for a referee export whatever its words), the
    validated export's, and `export_tag_missing` (Export scope, context `name`, DropExport;
    not beside `team_name_unknown`); `team_compiler`'s catalog row and a help paragraph.
    The catalog holds no hint text until Phase 8 (`messages.rs` module doc), so the rename
    hint is in the help. Test exports take `Midcup` by default (`co - X` → `co Midcup X`,
    924 lines, a names-only check over every changed file), since today's merge is
    `Midcup`'s and 4.34b gives `Full` new behavior; `Full` only where a scenario names it
    (TC-ID-01, TC-PLN-03 and 07). Nine fixtures renamed by the lead (`co Midcup Spring.zip`,
    `egg Midcup Tracer`, ...; `tracer/old/` keeps its old-layout name), the provenance
    script with them; the parity CPK unchanged. Gates green (152 of 252); `mutants-diff
    d1b9b5a`: 19 mutants, 15 caught, 4 unviable, 0 missed; `clef-diff`: 11 windows, no flag
  - [x] 4.34b `Full` rebuilds its team's kits, done 2026-10-07 (sidekick, landed first time;
    lead fix: the help paragraph rewrapped): `aesthetics_export` `validate/kits.rs`
    `missing_kinds` adds an empty `Kits/p1` (no surviving player kit) or `Kits/g1` to a
    `Full` team export, built by the same `kit_folder` as a real empty folder (the loop body
    moved out unchanged), so `all/` reaches it; the second kit drop in `validate_with` went
    (sidekick's contradiction, accepted: every folder drop precedes `kits::check`, and it
    would have dropped the empty `p1` standing for a dropped `p1/`). `team_compiler`:
    `UniColorBin::keep_kits` (sharing `edit_kits` with `set_kit`), the manifest's
    `full_team_kits` (team, kit numbers of its kit tasks, failed or not) reaching
    `CpkOutput::finish`, which keeps only those kits before the committed entries merge. Help
    paragraph. TC-BIN-14 (count 4, kit 2's base entry kept), 15 (`Midcup`, count 9), 19 (no
    `Kits/`: p1 and g1 placeholders, count 2), 20; TC-ID-01's export gains a placeholder
    `p1` (expected). A `Full` export holding only `all/` no longer reports
    `kit_all_unused`. Not yet: the `UniformParameter.bin` half (4.21, TC-BIN-16). Gates green
    (156 of 252); `mutants-diff 5683a02`: 23 mutants, 13 caught, 10 unviable, 0 missed; `clef-diff`: 27 windows, no flag
  - The rest lands with the steps that own the data: the team's stale kit configs removed
    from `UniformParameter.bin` and no FPC patching for a Full export, with 4.21
    (TC-BIN-16); a Full export's compiled players without boots or without gloves losing the
    matching installed row, with 4.22 (TC-BIN-17); the savefile fields, Phase 5; the upgrader's tag
    (`midcup` or `additions` in the old name), Phase 6; the two buttons on an untagged row,
    Phase 8.

- [x] 4.y `dds_convert` cache retention bound, moved to Phase 8 on 2026-10-07 (decision entry
  "the conversion cache's retention bound is built with the GUI's compile"): only a caller
  that keeps a `Converter` across runs accumulates conversions, and the CLI makes one per
  run. The step as written (found at 2.20d converge; spec `libs/dds_convert.md`
  "In-memory conversion cache", "Retention is separately bounded and budgeted"): the
  `Converter` holds every distinct conversion until `clear`; the pipeline's memory budget
  charges `retained_bytes` and evicts under pressure, and repeated edits do not keep every
  superseded conversion → verify: a test compiling the same export with one texture edited N
  times retains one conversion of it, and a budget smaller than the cache evicts rather than
  blocking a task. (Placed after 4.28: eviction is the budget's policy, which 4.28 gives the
  budget the accounting for.)

- [x] 4.c-pass **Clef full pass**, done 2026-10-07 at `53abc6f` (with 4.33's tree): 1,554
  windows, 68 flagged at 0.7, 54 merged flags, each ruled in `scripts/clef_rulings.md` from a
  read-only investigation per flag (evidence: the line's doc comment, the test pinning it,
  the plan): 53 rejected, 1 accepted (4.c-fix1). Three flags were test code in a file of its
  own (`#[cfg(test)] mod test_support;`, two `*_golden.rs`): `clef_scan.py`
  `test_only_file` now leaves such modules out (20 files, 57 windows), so a rerun reads
  1,497 windows, 51 flags, 0 new. Side findings fixed as comments: `legacy.rs`'s physique
  run is fourteen `i32`s, not thirteen. Not acted on: `ir/mod.rs:63` and `ir.md:53` say a
  root bone's `local_position` equals its global one while `ir.md:265` and the FMDL export
  test pin `[0,0,0,1]` (the two may describe import and export; unchecked).

- [x] 4.c-fix1 **`convert` keeps the target's base-copy flag when the source version has
  none**, done 2026-10-07 (sidekick, landed first time, no lead fix; Clef flag
  `pes_savefile/src/convert.rs`, ruled 2026-10-07): `PlayerEditFlags::base_copy` is
  `Option<bool>` (PES 15-18; `None` on 19-21), `PlayerEntry::get`/`set` gate it like
  `DribblingMotion`, `convert` carries it. The `team_toml` and legacy readers needed no
  change: both reach the model through the schema-gated `get`/`set`. Tests: a PES 19 source
  into a PES 18 template keeps its `Some(true)`; PES 18 to 18 copies both ways; the codec
  test checks the flag's presence on all seven fixtures. Gates green (156 of 252);
  `mutants-diff 6da1b6a`: 7 mutants, 6 caught, 1 unviable, 0 missed; `clef-diff`: 4 windows, no flag

- [x] 4.c-threshold **Clef threshold re-check** (after 4.c-pass, once
  `scripts/clef_rulings.md` holds 200 rulings or at this phase's converge, whichever comes
  first): the 2026-10-06 threshold table (`THRESHOLD` 0.7, chosen from 21 injected defects and
  203 reviewed windows) redone on real data. False alarms per threshold from the rulings' P and
  verdicts; missed defects from every defect found since by other means (reviewer, census,
  mutation run, in-game test) in code the scan had seen, its window's score read from
  `clef.out/cache.json`; and a sample of the full pass's windows scored 0.5 to 0.7, ruled, for
  what a lower threshold would add → verify: a decision entry keeping or moving `THRESHOLD`,
  with the table. Done 2026-10-09 (lead; `.tmp/4_y/clef_threshold.py` re-tiles the two
  scanned commits' production code in detached worktrees and reads the cached scores;
  record `.tmp/4_y/clef_threshold.md`): decision entry "the Clef threshold stays at 0.7":
  66 rulings, the 2 real flags at 0.71 and 0.77; 0 of the 10 highest windows under 0.7
  real; the one slip found since by other means (`own_package`, 4.y-fix1 (h)) scored
  0.07, its evidence outside the window; the census's three findings were missing rules

- [x] 4.y-fix1 **Converge small fixes** (from the lead's audit, 2026-10-09; one brief): (a)
  `pipeline` stops re-exporting `FALLBACK_AVAILABLE` (no consumer; the `pub` census); (b)
  `processing/texture.rs` `convert` releases the decode charge after the pass-through decision,
  since `keeps_blocks` makes the unwrapped copy the charge covers (Clef flag accepted
  2026-10-09); (c) TC-REF-04's test compares the written `incom_bsm.dds` with the converted
  marker, not only "a DDS that differs from the template's"; (d) TC-DEP-01's test snapshots
  `download/` and asserts nothing else changed (both from the verify re-run,
  `.tmp/4_y/converge_verify.md`); (e) `output/writer.rs` `commit_folder` asserts that an empty
  textures batch carries a dropping finding (the 4.17 "Open for converge" item: planning never
  plans an empty textures task, and the assertion says so); (f) a `.common` link to a `.ftex`
  gets the texture role a `.dds` link gets (`plan/roles.rs` `linked_texture_stem`; issue
  2026-10-07 below: a `.ftex` is an accepted texture format everywhere else); (g) the 4.34b issue
  settled by a test (a `Kits/all/kit.dds` no decoder reads, `p2/` inheriting it: is the kit
  compiled from the dropped file?) and fixed if red; (h) the 4.14e1 issue settled by a test (a Fox
  player holding `ingame_face`, his own `body.dds` and a combined `Boots/` folder's `body.dds` of
  other bytes: is the boots package dropped with `shared_texture_conflict`?) and fixed if red.
  Crates: pipeline, tc → verify: gates green; (c)'s test fails with one byte of the marker's
  converted bytes perturbed by hand; (f)'s test shows a `hair.ftex.common` link's model path
  pointed at the team's Common output.
  Done 2026-10-09 (sidekick, landed first time, no lead fix): (a) `pipeline` re-exports
  `memory_cap` alone; (b) the charge released after the pass-through test; (c) the marker
  entry equals `dds_convert::convert` of the source for PES 17; (d) `download/` snapshotted
  before and after; (e) `commit_folder` `ensure!`s a dropping finding on an empty textures
  batch, two unit tests (the writer tests' failure note now `DropFolder`, as a failed task's
  finding is); (f) green: `texture_format` already covers `.ftex`, the 2026-10-07 issue
  closed without code; (g) green: the deep pass drops each inheriting kit folder
  (`texture_type_mismatch [DropFolder] at Kits/p2 (file=Kits/all/kit.dds)`), the 4.34b issue
  closed; (h) red as expected, fixed: `own_package` answers the boots for an `ingame_face`
  folder on both engines, so the conflict is `merged_texture_conflict` dropping the player,
  the 4.14e1 issue closed. Red runs pasted for every test. Files: `pipeline/src/{lib,memory}.rs`,
  `output/writer.rs`, `plan/mod.rs`, `processing/texture.rs`, `tests/cli/{common_links,
  compile_exports,deploy,models,referees}.rs`. No `mutants-diff`: the whole-crate
  `team_compiler` run over this tree (4.y-conv) covers the diff.

- [x] 4.y-fix2 **An old-layout export is one finding; a dual-engine face diff is no conflict**
  (decision entries 2026-10-09, from the converge census): `export_layout_old` in
  `aesthetics_export`'s structure pass (`validate/root.rs`: a root folder named `Kit Configs`,
  `Kit Textures` or `Other`, case-insensitive; the export skipped with that one finding and no
  other structure finding computed; the catalog row, the hint naming the Export upgrader; the
  GUI needs nothing new), TC-ROOT-14; `xml_dif_conflict` narrowed in `deep/documents.rs`
  `face_diff_findings` (a `face.xml` `<dif>` beside a `face_diff.bin` is no conflict; on PES
  15-17 the bin is ignored with no finding, on PES 18-21 the xml is `xml_ignored_fox` as today),
  TC-XML-11, TC-XML-04 and TC-MOD-42 unchanged; plus one test each for the whole-crate runs'
  survivors, `kits.rs` `direct_metadata` (`&&`) and `pipeline` `memory.rs` `available_memory`
  (a bounds test on this machine); (e) three unit tests for `output/deploy.rs`'s surviving
  error-kind guards (`sweep_run`'s lock open, `probe_download`'s access-denied and in-use
  arms; two Windows-only). Three crates: GPT review (b) with the next `duck` batch.
  Crates: ae, tc, pipeline → verify: `check` on `C:/Data/4cc/Lab/Gud/EGG Aesthetics Export
  VGL26` prints one line, `Error export_layout_old [DropExport] (folder=Kit Configs)`, exit 1;
  `check` for PES 17 on `C:/Data/4cc/Refs/26_1-winter_refs/exports_to_add/refs for Winter 26
  Final Boss` reports no `xml_dif_conflict` (the census runner `.tmp/4_y/census/run_census.py 17`
  re-run shows the class gone). Done 2026-10-09 (Opus 5.5, landed first time; two
  contradictions accepted, see below): `validate_with` returns `export_layout_old` alone
  before any other rule (`OLD_LAYOUT_FOLDERS`, folded names; context `folder`), the
  `ISSUE_CODES` and `CATALOG` rows, a unit test and TC-ROOT-14; `user_face_xml.rs`
  `holds_face_diff_xml` counts a `face_diff.xml` alone (the pre-Fox writer already emitted
  the xml's `<dif>` and gave the bin no finding), the unit test flipped, TC-XML-11 proven
  with the `plain.bin` fixture, the help sentence corrected; the `direct_metadata` test
  (c). Contradictions: (d) the six `pipeline` survivors were not missing tests but the
  documented platform arms (`.cargo/mutants.toml`: each half ran the other platform's
  `available_memory` body), so the existing bounds test gained its 1 TiB upper bound and
  nothing else; (e) the three `deploy.rs` guards are killed on Windows by existing tests
  (`a_staging_whose_lock_cannot_be_opened_is_left`, TC-DEP-04, TC-DEP-02) and survived on
  the Linux half alone, so only the portable read-only test (406) was added; the sidekick
  also found the 4.14d-era note "the Linux half runs as root" wrong (`systemd-run
  --uid=debian`), corrected. Lead: the help chapter's `export_layout_old` sentence. Gates
  green (acceptance 279 of 280, TC-TEX-13 left for 4.y-fix3). Verify re-run by the lead with
  the release build of `d01a826` (`.tmp/4_y/census/run_census.py 17` and `21`, `check` over
  the 71 roots; results `results_17.tsv`/`results_21.tsv`, the pre-fix tallies kept as
  `*_before_fix2.tsv`): the 53 VGL26 exports print one line each, `Error export_layout_old
  [DropExport] (folder=Kit Configs)`, on both versions; `xml_dif_conflict` fell from 10 rows
  on PES 17 (the five referee folders of four exports) to none

- [x] 4.y-fix3 **Cube-map textures** (from the converge census: the referee robocopclassic's own
  `Common/env.dds`, a 128x128 DXT5 cube map with the bundled template's exact header, fails his
  folder at `compile` on both engines with `folder_pack_failed (error=env.dds: cannot convert:
  ... unsupported dds: cube map)`, while the template `env.dds` is emitted as it is). Recon
  done 2026-10-09 (sidekick, read-only; `.tmp/4_y/sk_fix3_recon_report.md`): both census files
  are byte-identical to the template; the compiler fails in `convert` → `decode_charge` →
  `dds_convert::probe` → `ftex::dds::read_layout`, before any engine branch; Red copies a DXT5
  cube map as it is on PES 15-17 and on PES 18-21 writes it through `ddsToFtex` as a type 0xD
  FTEX cube map, which `ftex::dds_to_ftex` already writes the same way; PES 17's own cube maps
  have the template's shape, PES 21's are FTEX types 0x5/0x7, and the 4cc Fox CPKs hold none.
  Ruling (DECISIONS 2026-10-09): a cube-map DDS is never decoded; PES 15-17 passes it as it
  is, PES 18-21 writes Red's FTEX cube map (a WESYS-wrapped one stays refused); whether the game
  draws type 0xD is a maintainer question. The step: `ftex::dds::is_cube_map` (`pub`, the
  compiler's), the route in `processing/texture.rs` `convert` before the decode charge,
  TC-TEX-13, the `dds_convert.md` sentence corrected. Crates: tc, ftex → verify: `compile --mode test` of `C:/Data/4cc/4cc aet compiler/Test_stuff/refs`
  for PES 17 writes robocopclassic's folder with `env.dds` byte-identical to the source.
  Done 2026-10-09 (Opus 5.5, one rework round): `ftex::dds::is_cube_map` (`pub`; the
  `DDSCAPS2` cube bit, or a DX10 header's cube flag), the route in `convert` before the decode
  charge (PES 15-17 the bytes as they are, PES 18-21 `ftex::dds_to_ftex(.., Normal)`, its
  error through `conversion_failure`), TC-TEX-13 proven on both engines (the template
  `env.dds`; PES 21's `env.ftex` equals `dds_to_ftex` of it, `is_cube_map`, type 0xD; PES 17's
  `env.dds` byte for byte). Contradictions, all five accepted: `ftex`'s unit tests live in
  `lib.rs`'s `tests` module (`dds.rs` has none); the scenario's folder holds no model (the
  tracer's Fox model converted for PES 17 adds 21 bone findings, so GIVEN was edited); the
  brief's DX10 `misc_flags` offset was 128+12, it is 136; `dds_to_ftex` read cube-ness from
  `DDSCAPS2` alone, so a DX10 file with the cube flag and no faces would have gone out as a
  one-face 2D FTEX with no error: the rework round made the writer read it by the same
  `is_cube_map` (such a file now fails as an incomplete cube map, test added); the plan's and
  the decision's "as it is, wrapped or not" contradicted their own next sentence (a wrapped
  cube map is not recognized), clause removed. Gates green (acceptance 280 of 280);
  `mutants-diff d7a63e4`: 21 mutants, 21 caught. Verify re-run by the lead with the release
  build of `213b329` (`.tmp/4_y/census/verify_fix3.py 17` and `21`, logs beside it): PES 17
  compiles the refs folder with exit 0, 15 files, robocopclassic's `env.dds` byte-identical to
  the source (before: `folder_pack_failed`, 0 files); PES 21 writes his `env.ftex` and the
  `folder_pack_failed` is gone, the folder's `skl_merge_conflict` (his Fox skeletons differ; the
  pre-fix census tallies already show it on three refs roots) remaining as before
- [x] 4.y-fix4 **S7's rework: a shared face's `.model` boots convert with their `.mtl`; a link
  combines with the player's effective parts** (the Astra review S7, 2026-10-09, rulings
  `.tmp/4_y/duck_rulings.md` S7.A1-1 and S7.A1-2). (a) On Fox a package's `.mtl` inputs are
  its own source's or the player folder's (`plan/mod.rs` `TaskKind::files`), so a linked face
  folder's `boots.model` reaches the boots task without its `.mtl` and `processing/model.rs`
  panics on the missing input: a `.mtl` is read by every package that reads a `.model` of its
  source. (b) `roles.rs` `link_combines` asks `holds_model`, which reads the player's own files
  alone, so a boots or gloves link does not combine with a linked face's boots/gloves parts or
  with his hand-split gloves while `item_rows.rs` prefers his exclusive package's id, and the
  link is worn by nobody (`player_folders.md` "A link plus local models combines",
  `hand_split.md` "Pipeline integration"): the link combines when the player's effective
  package of its kind has parts (own files, a linked face's files under his roles, the
  hand-split set), and the id pass answers the same. TC-MOD-58, TC-MOD-59, TC-MOD-60.
  Crates: tc → verify: the three scenarios proven, red first (58's red run is the panic).
  Done 2026-10-09 (Opus 5.5, landed first time; two deviations applied and accepted): (a) on
  Fox `TaskKind::files` reads a `.mtl` when the package reads a `.model` of its source
  (`model_sources`); pre-Fox keeps its two-case rule, since a pre-Fox `.model.common` link
  takes a local `.mtl` of its name though no package reads a `.model` of the folder
  (TC-MOD-24 panicked on the new rule; the defect cannot arise there, a face's `.model`
  being face content). (b) `has_effective_part` replaces `holds_model`: own files, each
  linked face's files under his roles, and on Fox the hands split off a face part
  (`is_hand_split`, taken out of `hand_split_parts`); `link_combines`,
  `link_feeds_own_package`, the id pass and validation's pool counts take the deep pass's
  `hand_weighted`. TC-MOD-58 (red run: the panic at `processing/model.rs:169`), TC-MOD-59
  and TC-MOD-60 proven; TC-MOD-60 uses the hand-split strip fixture as the face and as
  Crocs's gloves: the converted `.model` strip's hands and the tracer's gloves refuse to
  merge (`skl_merge_conflict`, `bone=sk_hand_l`), the fixture's bones being all roots
  where an authored glove's hand bone has a parent, a fixture artefact and not the
  scenario's point. Contradictions accepted: the converted boots take the bundled
  `body.skl`; `Faces/Round/skin.dds` added so the `.mtl`'s texture resolves and the test
  proves Round's `.mtl` was used (paths at the player's home); `link_combined` lines come in
  link file-name order; the deep pass cannot take the same answer (Issues). Gates green
  (acceptance 283 of 283); `mutants-diff 59af87c`: 45 mutants, 35 caught, 10 unviable, 0
  missed, the remote half's peak 8.99 GiB of the 9G cap with one build job (no build killed)
- [x] 4.y-fix5 **S8's rework: a Common model's local `.mtl` resolves its textures in the folder;
  every DDS kind the decoder refuses is `texture_codec_unsupported`; an FTEX portrait gets the
  full chain** (the Astra review S8, 2026-10-09, rulings `.tmp/4_y/duck_rulings.md` S8.A1-1,
  -2 and -4). (a) `processing/model.rs` gives every Common-linked model `PartTextures::Common`,
  so a Common `.model` converted with the player's local `.mtl` override (`mtl_search`: a
  name-matched local `.mtl` beats `Common/`'s) points its textures at the team's Common
  output, against `model_format.md` "Link files" (a stem resolves in the folder of the
  material file that set it): such a part looks in the folder first, then Common, as a
  `.model` with a `.mtl.common` does (`CommonSet`). (b) `conversion_failure` maps
  `ConvertError::Ftex(UnsupportedDds)` (a signed block format, a volume texture, an array, a
  paletted DDS, an incomplete cube map) to the ordinary task failure, so one such file in
  `Common/` drops every Common texture with `folder_pack_failed` where the catalog promises
  `texture_codec_unsupported` and the file alone: every DDS kind the decoder refuses is that
  finding (DECISIONS 2026-10-09). (c) `portrait` keeps an FTEX source's level count
  (`authored_mips`) where `player_folders.md` "Portraits" gives every non-DDS source the
  full chain: an FTEX portrait re-encodes with a generated chain like a raster one.
  TC-MOD-61, TC-TEX-14, TC-TEX-15. Crates: tc → verify: the three scenarios proven, red first.
  Done 2026-10-09 (Opus 5.5, landed first time; two deviations applied and accepted): (a) a
  Common `.model` whose resolved `.mtl` is not a direct Common file takes
  `PartTextures::CommonSet` (the deep pass raised nothing on TC-MOD-61, so
  `deep/materials.rs` is unchanged); (b) `conversion_failure` maps
  `ConvertError::Ftex(FtexError::UnsupportedDds(_))` with `ConvertError::Unsupported`; (c) the
  FTEX portrait arm drops the source's levels and its blocks too, since `encode_dds` emits a
  BC3 source's blocks as they are whatever their level count (the brief's two lines left the
  chain at one level: red run shown both ways), one `encode_dds` call for every format.
  TC-TEX-15's portrait is a 128x128 single-level BC3 FTEX, not `small_dds()`'s 12x12 (a
  portrait's sides must be powers of two: `texture_not_pow2`); the scenario says so now.
  Brief errors it found: a Common texture's finding is scoped `at Common (file=..)`, pinned
  by a unit test already; the tracer has no `face_high.fmdl` (`copy_tracer_face` used);
  `tracer_kit()` is a 16x16 DXT1 with five levels (TC-TEX-14's wording corrected). Observed,
  unchanged: the deep pass checks a player's own `.mtl` against his folder's stems alone, so
  a local `.mtl` naming a stem only `Common/` holds gets `mtl_texture_not_found` (Keep)
  while the compile points it at Common's copy, as before this step. Gates green
  (acceptance 286 of 286); `mutants-diff 327a5ce`: 6 mutants, 5 caught, 1 unviable, 0 missed
- [x] 4.y-fix6 **A DX10-header DDS portrait goes out re-headered** (the VGL stream's PES 19 crash
  of 2026-10-09: a member's `player_XXX11.dds`, a 128x128 single-level BC3 under a DX10 header
  with the sRGB id DXGI 78, crashed the game when its slot was hovered; the same blocks under a
  `DXT5` header, resaved with paint.net, did not. `processing/texture.rs` `portrait` ships a DDS
  source byte for byte (reproduced with the release exe on PES 17 and 21, `.tmp/lead/
  portrait_probe.py`), the only place a raw member DDS reaches the game: pre-Fox textures go
  through the converter, which rebuilds the header, and Fox textures become FTEX. Rule
  (`player_folders.md` "Portraits", DECISIONS 2026-10-09): a DX10-header DDS portrait is
  re-headered with `ftex::dds::header_bytes` for its format (legacy FourCC for BC1-3, masks for
  BGRA8, DX10 UNORM for the rest), data untouched, reported `portrait_header_rewritten` (Info);
  an uncompressed DX10 layout with no `PixelFormat` (RGBA8) takes the raster route. Evidence:
  `.tmp/lead/census_dds_headers.out` (1218 cup portraits: 520 single-level legacy, 12 DX10 BC7,
  no DX10 sRGB); which header property the game refuses is a maintainer in-game question
  (`QUESTIONS.md`, variants in `C:/Data/4cc/Tools_Mine/temp/variants/`). TC-PRT-04. Crates: tc
  → verify: TC-PRT-04 proven, red first (today the file goes out byte-identical).
  Done 2026-10-09 (Opus 5.5, one small rework round): `portrait` takes the task's findings;
  a DDS source is unwrapped and its layout read (`read_layout`, errors through
  `conversion_failure`); the legacy header goes out as it is, wrapped or not; a DX10 header
  over a `DdsPixel::Format` with tightly packed rows is rebuilt with `header_bytes`, the data
  appended, `portrait_header_rewritten` (Info, Keep; `file`, `dxgi` read at offset 128); an
  uncompressed DX10 layout, or one with a declared row pitch (`header_bytes` declares tight
  rows, so re-headering padded data would write a wrong file: the rework), takes the raster
  route. `messages.rs` four tables. The finding is scoped to the portrait file, as every
  portrait finding is (the brief said the folder). Brief errors it found: `just gates` has no
  catalog scanner, the in-crate test wants `Code::ALL` order; `wezlib::decompress_if_wrapped`
  returns a `Result`. Observed, unchanged: the 12 cup BC7 portraits (DX10, UNORM id) get the
  Info too, the plan's one rule; a wrapped DX10 portrait goes out unwrapped. The maintainer
  identified the source: paint.net's "BC3 (sRGB, DX 10+)" menu item writes this header,
  "BC3 (Linear, DXT5)" the legacy one; the cup's BC7 portraits are its "BC7 (Linear, DX 10+)".
  TC-PRT-04 proven (PES 21 then 17), plus a unit test of the raster route. Gates green
  (acceptance 287 of 287); `mutants-diff d368c8e`: 9 mutants, 8 caught, 1 unviable, 0 missed

- [~] 4.y-conv **Converge** (`AGENTS.md` "Closing a phase" (1)): the lead's audit of
  `team_compiler`, `aesthetics_export`, `pipeline` and the Phase 4 edits of the lib crates
  against `development_plan.md` "Phase 4", the `pipeline.md` walkthrough, `messages.md`,
  `settings.md` and every TC ID; `just acceptance strict`; the design-health pass (design-tell
  sweep, `just mutants` per crate with every survivor triaged, `just clef` per crate with every
  flag ruled, `pub` census); the census over
  every export on the maintainer's machine (Everything index) run through `check` and `compile
  --mode test`, tallied by outcome; then the cross-family reviewer loop, one surface per crate
  (queued per the handover if no reviewer is available); each gap a new step above this one →
  verify: `just gates` green, `just acceptance strict` green, the census tally recorded in the
  worklog with every failing class diagnosed.
  The lead's audit, done 2026-10-09 (notes `.tmp/4_y/audit_notes.md`): plan shapes against
  the code (`core/architecture.md`'s `StudioTool`, `CliError`, `AppPaths`, the event types:
  no deviation; `pipeline.md` "Run driver shapes (Phase 3)" is rewritten from the code at
  4.z-rewrite); every step's verify criterion re-run by a sidekick (`.tmp/4_y/converge_verify.md`:
  64 rows, 55 proven by a green test, 3 recorded, 1 superseded, 1 deferred, 3 without a
  test that asserts the clause (4.19f, 4.24, both given their assertion at 4.y-fix1; 4.31's
  Red rebuild, run by the lead 2026-10-09 in the 2026-10-07 Red copy over the fixture's `old/`,
  `.tmp/4_y/red_rebuild_compare.log` and `red_rebuild_nested.log`: every entry byte-identical
  to the fixture's `red/`, the nested face CPK's twenty entries included, the nested CPK
  itself differing in four bytes near its end, its write timestamp), 1 stale criterion
  re-read, 4.25a); `just acceptance strict`: TC-OUT-18 was the one
  unproven scenario and got the lead's test (`tests/cli/bins.rs`); the catalog's producers
  (`.tmp/lead/messages_producers.py`: 25 rows with none, 23 deferred by the plan, 2 withdrawn,
  decision entry); the design-tell sweep over the whole Phase 4 diff (67,692 added lines,
  `.tmp/lead/sweep_phase4.py`: no new trait, every `drop(` a permit or handle, the wildcard
  arms on strings and slices, two inherent bulk copies commented); the `pub` census
  (`.tmp/lead/pub_census.tsv`: 2 + 63 + 16 items, one orphan, 4.y-fix1 (a)); Clef over the
  three crates (529 + 108 + 11 windows, 6 flags: 1 accepted, 4.y-fix1 (b), 5 rejected in
  `scripts/clef_rulings.md`); the census over 71 export roots found through Everything
  (`.tmp/4_y/census/`: the 53 VGL26 old-layout exports of `C:/Data/4cc/Lab/Gud`, 16 referee
  exports, 2 older team exports; `check` and `compile --mode test` on PES 21 and PES 17):
  the 55 old-layout exports each `export_tag_missing` plus a wall of structure findings
  (decision: `export_layout_old`, 4.y-fix2); of the 16 referee exports 3 compile clean on
  both engines, the rest fall into eight classes, each diagnosed in the issues below
  (two changes: 4.y-fix2's dif rule, 4.y-fix3's cube maps; the others the plan's rules, a
  true export defect, or a maintainer question). The 4.17 "Open for converge" items ruled:
  the empty textures batch gets its assertion (4.y-fix1 (e)); `bone_matrices`, the `.model`
  collar off its skeleton and the IR's mesh tags are `model_convert` work deferred to Phase 7
  (the tag question to the maintainer); the egui link (sidekick report
  `.tmp/4_y/sk_egui_report.md`): egui is not what hit the VPS cap. The 4.14d kills were
  per-mutant rebuilds (1 to 2.3 GiB each on the PC), and the host's `/tmp` is a 7.8 GiB tmpfs
  (checked 2026-10-09), so cargo-mutants' two build copies, `target/` of 4 GiB each with
  egui linked, sat in the unit's memory; `scripts/mutants.py` now builds the remote half under
  `~/studio-mutants/tmp` on disk (`TMPDIR`), and `team_compiler` leaves `LOCAL_ONLY_CRATES`
  (whole-crate runs 2026-10-09, split: `aesthetics_export` 548 mutants in 4.5 min, 2
  survivors: `validate/mod.rs:197` `==`/`!=` equivalent (an undecided root is `DropExport`,
  the empty slot map's variant unobservable) and `kits.rs:268` `direct_metadata`'s `&&`, a
  missing test, 4.y-fix2 (c); `pipeline` 72 mutants in 2 min, 6 survivors all
  `available_memory`'s platform reads, one bounds test, 4.y-fix2 (d), and 8 timeouts all
  `MemoryBudget::acquire` waits a mutated comparison makes endless, caught; `team_compiler`'s
  run is taken after 4.y-fix1, over its tree: the first attempt died with the console the
  lead's wrapper lost (every build 0xC0000142, filed unviable), the second started while the
  fix1 sidekick was editing the crate, which a run copies at its start, and was stopped)
  so its whole-crate run below proves the split works again (the run fails listing any
  killed build). Taking egui out of the test binaries would need the plan's `StudioToolUi`
  trait split (a feature gate is non-additive and leaves `studio_core`'s own tests broken)
  for a 1.4 GiB smaller cold build; not done: the cap problem was the tmpfs. The lib-crate
  split of `face_diff`, `user_face_xml`,
  `kit_variants`, `mtl_search` is not done (each a crate-boundary move for design health
  alone, with the compiler their only consumer; revisit when a second consumer appears);
  `xml_model_not_found` for a converted Common model named by a member's xml waits with
  Phase 7's conversion work; `pes_model`'s 0 positions on the stock `referee_collar_026.model`
  stays an open issue for the next `pes_model` census. Still to run: `just mutants` over
  `aesthetics_export`, `pipeline` and `team_compiler` (six hours, local), the `duck` reviews.

- [ ] 4.z-rewrite **Rewrite** (`AGENTS.md` "Closing a phase" (2) and (3)): `development_plan.md`
  "Phase 4", `pipeline.md` (the walkthrough's Phase 4 parts and the "Run driver shapes" block,
  which becomes the Phase 4 shapes), `messages.md`, `settings.md`, `object_model.md` "Validation
  semantics" (deep pass), `team_compiler/README.md` "Development phases" and the "Phase 3 scope"
  paragraph (its refusal list replaced by what Phase 3 built) in the present tense; resolved open
  questions moved to "Resolved decisions"; the worklog's step list collapsed to the phase row →
  verify: `rg -n "Phase 4" docs/plans` shows no future-tense Phase 4 sentence outside deferred
  items; the worklog's Phase 4 heading is one row plus any `manual: checked` lines

Out of Phase 4 (named so the steps above do not absorb them): the savefile step, the aesthetics
patch and every `savefile_*`/`patch_written` code (Phase 5, `output/savefile.rs`);
`PlayerAppearance.bin` rows and the stock/default-ID boots and gloves rows from `settings.toml`
(Phase 5); the other `settings_*` codes and `fpc_strip_conflict` (Phase 5, resolved values; Phase
4 reports only `settings_toml_invalid`); `run_pes` and the Launch PES button as one launcher
(Phase 8); `FolderStatus`, `Progress` and `Complete` events, live validation (`check/`,
`watcher.rs`) and every GUI action on a degraded run (Retry, Open output folder, Relaunch as
administrator, Upgrade DpFileList dialog): Phase 8; glTF sources: Phase 7; the GPU BC7 backend:
16.x; the conversion cache's retention bound and its charge (4.y): Phase 8, with the first
caller that keeps a `Converter` across runs; the parity run over upgraded reference
exports: after Phases 5 and 6
(`development_plan.md` "Phase 4" Verification).

Phase 4 open questions (maintainer):

- The sock table's look in-game (4.10): `KIT_LAYOUT_REMAP` approximates, in two bands, a map
  the models give to within about 10 px of 2048 (`pipeline.md` "Layout conversion"). Test 3
  (`.tmp/4_0/apptest/out_test3/GUIDE.txt`, 2026-10-05) is ready for it. To
  settle: a pre-Fox kit with a design on its socks (hoops do not show it; a vertical stripe
  or a logo does), compiled for a Fox game with the `pre-fox` marker and looked at in-game.
- Collars beyond the stock set (4.9, 4.27; the maintainer's idea, 2026-10-03): PES 15 loads
  `collar_105`, which its stock game lacks, so the games probably accept collar IDs they do
  not ship. If so, the FPC collar and the referees' marker could move to IDs no stock collar
  uses (still reserved), and a team could replace or add any ID nobody else claims, which
  would retire `collar_id_invalid`'s stock-set rule. What is known: a kit config stores each
  collar in one byte (`kit_config` offsets 0x14 and 0x15), so a kit can name 1-255 at most,
  whatever the file name allows; the only non-stock collar in any install is that PES 15
  one, an empty 852-byte model, so nothing shows yet that a Fox game (18-21) loads one, or
  that a visible model at a non-stock ID renders. To settle it: an in-game test per engine,
  a kit naming an ID above the version's stock set (200, say) with a visible collar model
  under that name. Until then 4.9 keeps the stock-set rule and the reserved 105 and 77.
  Test 3 (`.tmp/4_0/apptest/out_test3/GUIDE.txt`, 2026-10-05) is ready for it: collar 200 on
  PES 21 and PES 17, a copy of each game's collar_107. Edit mode shows only kit 1 (and the
  GK kit), so Test 3b (`out_test3b/`, 2026-10-06) puts kit 2 (sock stripes B, collar 107)
  on kit 1 as a second load; collar 201 is on the GK kit in both games. The maintainer
  postponed Tests 3 and 3b (2026-10-06); this applies to the sock item above too.

Answered 2026-10-03 (decision entries of that date; each is in the plan): the official
DpFileList's entries (`resources/templates/DpFileList.txt`, the maintainer's own edit of the
lead's draft: 53 entries, the test CPK `4cc_99_test`) and the check of the installed list on
every compile (`dpfilelist_not_official`); the `colors.txt`
grammar (one color per line, in both files); the kit icon as a marker file and the marker
spelling (4.30); the Fox referee marker as a reserved collar instead of a `dt00_x64.cpk` write
(4.27); the default `cpk_name` on PES 21 (intended: that install's list is obsolete, and the
upgrade to the one standard list is the path); one official `DpFileList.bin` for every version
(4.25), whose 53 entries the game loads whole (the maintainer's in-game tests on PES 2015 and
PES 2021: each install's own list extended to 53 entries with placeholder CPKs, the test CPK
last, `.tmp/4_7/dpfl_pes15_53.py` and `.tmp/4_7/dpfl_pes21_53.py`; the versions between are taken to
hold it too); `face_diff.xml` decoded with the `base64` crate (4.5d); a model's missing texture is an
Error, with the installed CPKs searched for a partial export's Common textures (4.29), and
`fmdl_no_texture_ids` is dropped; a format finding drops its folder by severity (4.7). The
pre-Fox parity reference was never a question: it is the lead's step 4.31. The kit layout
referee marker (4.27) waits on nothing: its models are the 4cc's `referee_prop.fmdl`
(`C:\Data\4cc\Tools_Mine\4cc refs compiler\referee_prop.fmdl`, texture `cup_logo.dds`
beside it) and Red's pre-Fox referee template's `referee_prop.model` with its `.mtl`
(`Engines/templates/refscpk_prefox/common/character1/model/character/parts/referee/`), and
the square stays on the ground as a collar by static painting (`blue_port.md` "Referee
export processing"). The far-vertex rule stands as it is (4.7): the lead's census of the
VGL26 corpus (2,938 FMDL, 21 `.model`; `.tmp/4_7/fmdl_census/corpus_census.txt`, `far.txt`)
found one Error class, `fmdl_vertex_far_from_origin`, in 89 `face_high`/`hair_high` files of
7 exports, every vertex of each parked 14,660 units or more from the origin, and those
parked placeholder faces and hair are the main cause of the matchday lag the rule exists
for, so 4.7 drops their folders. The kit layout
table (4.10) takes its numbers from the games' uniform models alone: the pair made with PES
Master's two kit creators (scripts, renders and `FINDINGS.md` in `.tmp/4_7/kit_creator/`) agrees
with them on the socks and not on the shorts, and the models win (`pipeline.md` "Layout
conversion").

### Phase 5 — Savefile integration

Steps are itemized when Phase 4 closes; one is fixed already:

- [ ] 5.0 Check the latest commits made to the unofficial 4ccEditor fork (`Tools_4cc/4ccEditor-1`,
  origin `AnonymousClouds/4ccEditor`; `pes_savefile/README.md` names it the reference): it is
  under constant development. Fetch, then read every commit after `4a95b7c` (2026-06-19, "Add
  Tactics tab for 16-21. Add ability to import Texports from 15-21"), the local clone's head at
  Phase 2's close → done when: each savefile-relevant
  change (schema fields, write behavior, `.4ccs`/`.4cct`/Texport formats) is reflected in
  `pes_savefile` and its plans or recorded as not applying, before any Phase 5 writing code, and
  the commit reached is in the log

### Phase 16 — Polish and distribution

Steps are itemized when Phase 15 closes; one is fixed already:

- [ ] 16.x `dds_convert` GPU BC7 (deferred from 2.5b to Phase 4, decision entry 2026-09-21, then
  past Release 0.1.0, decision entry 2026-09-28; spec `libs/README.md` "First-release desktop GPU
  BC7"): `block_compression`'s wgpu backend on a Vulkan/Metal device, CPU fallback with the
  fallback reason reported, cold pipeline creation and upload/readback measured first, then the
  texture step's bounded batches → verify: GPU and CPU outputs decode within the same tolerance
  on the `dds_convert` fixtures; the fallback path exercised by forcing no adapter

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history. A question only the maintainer
can answer is not an issue: it goes to `docs/QUESTIONS.md`, and an issue that waits on one
points there.

- open (converge census, 2026-10-09) — a pre-Fox face of several `.model` parts converted
  for PES 18-21 fails to merge: each part's conversion generates its own skeleton (the bones it
  uses) and the face merge refuses the differing skeletons (`skl_merge_conflict`,
  `skeleton=differs` or a `bone=`), nine referee folders in seven of the cup's current referee
  exports (FAA, germanbro, coolguy, geodude, donte, ...). Red never converted a `.model` for
  Fox, so these referees were never on PES 21; the fix is the converted parts of one folder
  sharing one generated skeleton (the union of their bones, or the target's full table), a
  `model_convert` and `fmdl` merge question for Phase 7, when the conversion work resumes.
- open (converge census, 2026-10-09) — on PES 18-21 a member's own `face.xml` is ignored
  (`xml_ignored_fox`), so a pre-Fox referee folder with Red-packed names
  (`oral_fixed_kit_win32.model`, five `.mtl` files) converted for Fox pairs each model with
  the `.mtl` search's choice (`apc.mtl`, the first by name) where the xml names `fixed.mtl`, and
  `model_material_undefined` drops the folder (ANF, four cup exports). The xml's `material`
  attributes could drive the conversion's pairing on Fox; a plan ruling first. Low priority:
  these referees are pre-Fox material.
- open (converge census, 2026-10-09) — two reserved subfolders holding one texture under one
  stem with identical bytes (`boots/outline.dds` and `gloves/outline.dds`, the maintainer's
  `refs fox test` and `refs lan party` exports) are `texture_stem_conflict` and drop the
  folder, where `merged_texture_conflict` deduplicates identical bytes. The conflict is the
  structure pass's (no bytes read there); none of the cup's real referee exports does it.
  Revisit if one does: the deep pass would withdraw the finding for identical bytes.
- open (converge census, 2026-10-09; diagnosed as the plan's rules, no change) — the census's
  other refusals of the cup's referee exports: `common_file_disallowed` on `Common/refkit/`
  (a maintainer question, `docs/QUESTIONS.md`); `file_type_disallowed` on files two levels
  down (`Face/kit/`, `Face/face/`, a `Common_shared/` subfolder, a `.model` in a player's
  `common/`), which the plan's reserved-subfolder rule names and the Export upgrader flattens;
  `texture_not_pow2` on a 192x512 mipped texture (a maintainer question); ANF's
  `face_high_win32.model` naming `head_phong`, which the `face.mtl` his xml names lacks (a true
  defect of the export; `pass_through` compiles it as Red did). Three referee exports compile
  clean on both engines (Summer 26, `refs numbered`, Autumn 25 Final Boss on PES 17).
- open (found at 4.14b's review) — pre-Fox faces: two files of one face packing under one
  name (`hat.model` beside `face/hat.model`, two `.mtl` of one name in the folder and
  `face/`, or a member's own `dummy.mtl` when the dummy is added) fail the face task at
  `compile` (`folder_pack_failed`, folder dropped) while `check` reports nothing. No finding
  code exists for it; a planning check naming both files would make `check` agree. No real
  export is known to do it.
- open (found at 4.14b's review) — the pre-Fox gate names an `ingame_face` marker as
  `<folder>/ingame_face` (without a `.txt` it may carry), since `PlayerFolder` keeps no
  marker file's name; the message is still findable. Gone when 4.14e compiles it (links
  compile since 4.14c1).
- open (found at 4.14c1's review) — pre-Fox combined face: the player's own texture of a
  stem the linked `Faces/` folder also holds, with other bytes, is `merged_texture_conflict`
  and drops the folder, as on Fox, while his own model or `.mtl` of a shared file's packed
  name replaces it (the plan's copy with local files on top). On pre-Fox no merged model
  makes the texture ambiguous, so the player's could win too; the plan's step 6 rule is
  engine-wide today. Safe as it is (an Error, nothing wrong written).
- open (found at 4.14c2's review) — pre-Fox Common: a texture link whose target only an
  installed CPK holds is still `common_link_missing` on PES 15-17
  (`installed_common_textures` is empty there; the Fox walk's stems are `.ftex` paths). A
  later step can read the installed CPKs' `common/character1/.../common/{team}/` `.dds`
  stems the same way.
- open (found at 4.14c2's review) — pre-Fox Common: with `pass_through` on, a `Common/` `.mtl`
  kept despite an eligible Error is left out of the deep pass's `.mtl` search (which cannot
  see the setting), so a model whose only `.mtl` it is gets `model_material_undefined` where
  `compile` could have built it. Conservative (drops, never panics); rare.
- open (found at 4.14c2's review) — pre-Fox: two `.model.common` links to one Common model in
  one folder (`legs.model.common` and `boots/legs.model.common`) give one `face.xml` entry,
  the first in file order, though their positions could type them differently. Fox merges
  them as one part too. No real export is known to do it.
- open (found at 4.14c1's review) — pre-Fox: a file a task does not use is still read and
  charged (a shared face's model or `.mtl` the player's file replaces, a shared boots
  folder's `.mtl` its model does not use): `TaskKind::files` lists every role. Harmless.
- open (found at 4.14b's review) — pre-Fox faces: every `.mtl` of a player folder is packed
  into the face, one no model uses included (a blank face too), as Red packs the whole
  folder. Harmless to the game; revisit if a census shows stray `.mtl` files are common.
- resolved (2026-10-09, 4.y-fix1 (h): confirmed by a red test; `own_package` now answers the
  boots for an `ingame_face` folder on both engines, the conflict `merged_texture_conflict`
  dropping the player as the plan says; the pre-Fox gloves case is the same rule) — Fox,
  `ingame_face`: `ModelFolder::own_package`
  still answers the face for a Fox player holding the marker, so his own textures count for a
  face he does not have when a stem conflicts. A stem his folder and a boots folder his link
  combines both hold may then be `shared_texture_conflict` dropping the boots, where the plan's
  `merged_texture_conflict` (or identical bytes deduplicated) looks right. Needs a test before
  any fix. Pre-Fox has the same shape since 4.14e2: a marked player's own textures count for
  his boots, so a combined `Gloves/` folder's texture of one of his stems with other bytes may
  drop his whole gloves package although his own parts use his own texture.
- open (2026-10-08) — the PC blue-screened three times during a local mutation run (4.14c2's
  half, 4.14e1's and 4.14e2's; MEMORY_MANAGEMENT at the first), never otherwise: the two
  cargo-mutants jobs each build a copy of the tree, the heaviest load the machine gets now that
  `team_compiler` runs locally whole. The maintainer memtested the RAM (clean), turned XMP off
  after the second, and after the third set the paging file to 32 GB, the size it had before
  the crashes began. A killed run leaves its two `%TEMP%\cargo-mutants-4cc-studio-*.tmp`
  copies (3.6 GB each; `.tmp/lead/rm_mutant_copies.py` removes them, clearing the read-only flag
  a test leaves on one file). If a run crashes again at 32 GB, the next lever is one job
  (`--jobs 1` in `scripts/mutants.py`'s local config).
- open (2026-10-08, 4.14e3's plan) — pre-Fox: a hand-weighted model behind a `.model.common`
  link is listed whole by its Common path, not split (`hand_split.md` "Pipeline
  integration"): the game then loads a body with hands on the body skeleton. Splitting it
  means packing a copy into the face. Build it when an export has one. Likewise (4.14e3,
  decision entry) a hand-weighted model whose `.mtl` is a Common file fails its folder with
  `model_conversion_failed` instead of reading that file for the split.
- open (found at 4.14e2's review) — pre-Fox combined folders: when a player's `.mtl` replaces
  a linked folder's of the same name and a linked model he does not replace used it, that
  model is listed with his `.mtl`, which the deep pass never compared with its materials
  (it checked the linked `.mtl`): a material his lacks is undefined in game with no finding.
  The combined face behaves the same way. Rare (two `.mtl` files of one name, the linked
  model kept); a fix checks the material names against the `.mtl` that is packed.
- resolved (2026-10-07) — the remote half reached its 9 GiB cap with `REMOTE_BUILD_JOBS`
  already at 1 (4.14c1 peaked at 8.99 GiB; 4.14d's killed two builds, rerun locally). The
  maintainer chose to run `team_compiler`'s mutants on the PC only: `LOCAL_ONLY_CRATES` in
  `scripts/mutants.py`, honored by `just mutants` and `just mutants-diff`.
- resolved (2026-10-07, 4.14d) — `model_material_undefined` covered only "no `.mtl` found";
  the deep pass now also reports a material name missing from the paired `.mtl`.
- open (found at 4.18's review) — hand auto-split: a split glove beside an authored glove
  of the same hand may not merge. The split prunes the bones no vertex of a glove uses, so
  a split `sk_hand_l` can lose its parent `sk_forearm_l` where an authored glove keeps it,
  and `fmdl`'s merge reports `skl_merge_conflict`, leaving the gloves out (seen with the
  tracer's glove beside the lead's fixture, whose bones are all roots; unmeasured on a real
  full-body model). A player holding both is unlikely; revisit if one appears (the fix
  would be the split keeping a used bone's ancestors, a `model_convert` change).
- open (found at 4.18's review) — hand auto-split: a player folder with a plain `Gloves/`
  link and a hand-split face part keeps the link plain (links combine by file names), so
  the shared folder still compiles under its shared ID while the player's `GloveList.bin`
  row goes to the split gloves, its own task. An authored glove beside a link combines
  instead. The plan is silent; no real export is known to do it.
- open (found at 4.18's review) — hand auto-split: a face model whose hand-weighted
  vertices sit in no face gives `model_hand_split` an empty `gloves` value and an empty
  gloves package. No real file is known to reach it.

- resolved (2026-10-07, by rule: decision entry "the refs CPK beside the team side") (found at
  4.19a's review) — a refs export's `.common` texture link is never satisfied
  by a texture already installed under `common/999/sourceimages/`: `validation.rs`
  `installed_common_textures` finds the team's id through the teams list, which has no
  `/refs/` row, so the set is always empty for referees. Since 4.19b's walk passes over the
  installed refs CPK, the only such texture would be one the game itself ships under 999;
  settle at 4.19b whether the lookup should use `REFEREE_TEAM_ID` or stay empty by rule.

- open (found at 4.26c's review) — when an all-or-none install's undo cannot move an old
  CPK back from `.cpk.old` (`deploy.rs` `undo`), the failure is only logged: `download/`
  then lacks a CPK its list names, so the game loads none of the folder, and the member
  sees only the `old_cpk_locked` or `deploy_target_unwritable` that started the undo. The
  move back renames a file the run itself just moved, so nothing real is known to fail it;
  the plan names no finding for it. Revisit if a member meets it.

- open (found at 4.26b's review) — a multi-CPK run promotes its CPKs to `output/` one
  rename at a time, so a rename failing part-way leaves `output/` with some new CPKs and
  some of the previous run's. The plan settles all-or-none for the installation only
  (`pipeline.md` "Deploy CPKs"); the output folder is the maintainers' build folder, and the
  next run rewrites every CPK. Revisit if a maintainer meets it.

- resolved (2026-10-07, 4.21c: the bins are parsed as they are read) — an installed bin the walk reads but that does not parse (a
  `UniColor.bin` whose length is not whole records, a corrupt `UniformParameter.bin`) fails
  only in `CpkOutput::finish`, after the exports are processed, as `cpk_write_failed`, not
  as `installed_bin_unreadable` before any export is read; `uniparam_compile_failed` is not
  wired either. The plan is silent on whether parsing is part of "cannot be read". Settle
  with 4.21c, which reads the installed `UniformParameter.bin`'s entries anyway.

- resolved (2026-10-09, 4.y-fix1 (g): the deep pass reports an `all/` texture's finding on each
  inheriting kit's scope with `DropFolder`, so the kit is dropped, never compiled from the
  file; a CLI test pins it, no code changed) — `aesthetics_export`
  `validate/kits.rs`: a kit's textures, its own and those inherited from `all/`, are never
  filtered against the dropped files (`validate_with` filters a kit's `colors.txt` only),
  so a kit texture a content finding drops as a file would still reach the kit, and a
  kit inheriting it would compile from it. Whether it is reachable depends on how the deep
  pass drops a kit texture that does not decode (the file, or its kit folder); settle that
  with a test (a `Kits/all/kit.dds` holding bytes no decoder reads, a `p2/` inheriting it)
  before fixing.

- open — the `livecpk/` replacement is not atomic (4.23a): the previous tree is removed before
  the staged one is renamed in, so a copy that fails partway (the output folder on another
  drive) or a removal that fails partway (a file the runtime holds open, unverified for FoxDen)
  leaves `livecpk/` partial or empty, reported as `output_commit_failed`. A swap through a
  sibling folder would narrow it; decide with the first in-game sideload test.
- resolved (2026-10-09, 4.y-fix1 (f): the premise was false; `texture_format` goes through
  `SourceFormat::from_extension`, which covers `ftex`, and a CLI test now pins the link's
  role) — a `.common` link to a `.ftex` (`hair.ftex.common`): the lib counts any texture type as a
  texture link, but planning's `linked_texture_stem` gives a role only to the formats
  `dds_convert` reads, so such a link gets none and the model's path is not pointed at the
  Common output (seen at 4.29b, older than it; not tested). Decide whether a `.ftex` link is
  refused or read.
- open — the compiler reports only two of `kit_config::validate`'s findings (the two a
  version's encoding clamps, as `kit_config_version_clamped`, step 4.9a). The others have no
  row in `messages.md`: `kit_collar_zero` (the lib's one Error; `messages.md` says
  `kit_config_invalid` covers a config that "fails to validate", which the deep pass does
  not do), `kit_shirt_model_unknown`, the three "requires model 144 or 160" warnings and
  `kit_unknown_sleeve_value`. Before a kit is dropped for a zero collar, count how many real
  configs carry one (the old-format exports' `Kit Configs/` and the installs' CPKs).

- open — the headless egui test harness (a frame with AccessKit on, a node by label, a click as
  press and release frames) exists twice, in `studio_core/src/shell/mod.rs` and
  `team_compiler/src/view/mod.rs` (3.y design sweep). The next tool view in Phase 8 would be
  the third copy: extract it then into `studio_core` behind a test-support feature, rather
  than a third copy.
- open — for Phase 8's settings menu, the first code that saves `settings.toml`: the `[common]`
  table is read into `CommonSettings`, so a key it does not know (a newer version's) is lost on
  save, while a tool's table keeps unknown keys (`core/gui.md`). Settle whether `[common]` keeps
  them too before that save path lands (review S2.1).
- open (gloves only) — the boots/gloves rows a `Full` export leaves (`pipeline.md`
  "Bins accumulation", TC-BIN-17, decision "a team export's name carries `Full` or `Midcup`").
  Phase 4 (4.22) drops the installed row of a compiled player the export gives no boots or
  gloves, so he has no row until Phase 5 writes the default-ID rows. A team player the `Full`
  export does not compile at all keeps his rows as they are (decision 2026-10-05; the plan
  already says so). Test 2 (PES 21, `.tmp/4_0/apptest/out_test2/`, run 2026-10-06), same results
  on the 4cc exe and the stock exe:
  - **Boots:** a stripped player with no `BootsList` row wears the boots-0 model (plain black
    boots), not his save's boots field (1076). So a dropped row and a row naming 0 look the
    same, and 4.22's drop is safe for boots.
  - **Rows are read:** stripped players with a row wore k0629 (/gd/ boots).
  - **Kept appearance id:** the control (his own boots 1076, plus a k0629 row) wore his own
    boots, so a kept appearance id beats the row on PES 21.
  - **Gloves:** the 4cc exe shows bare hands on both goalkeepers (its gloves patch). On the
    stock exe a `GloveList` row is read (g0003, yellow). The no-row goalkeeper wore white
    gloves, which are both his save's field (g0001) and possibly the default: one with
    non-1 save gloves would tell the two apart. Settle before 4.22 only if the gloves default
    matters.
- resolved (2026-10-05, maintainer) — the PES 16 Common patch is made when the cup next plays
  PES 16, and the plan treats it as existing; `kitN` works on Fox through FoxDen and comes to
  pre-Fox when the cup returns to it (`messages.md`, `model_format.md` "Kit-dependent
  assets").
- resolved (2026-10-05, maintainer) — generated `settings.toml` files and names: names are
  used only from an export with the `autopilot` root marker, so generated names do no harm
  (`settings_toml.md`, `operations.md` "Aesthetics patch").
- answered (2026-10-05, below) — the seed rows and multi-CPK mode: strip-and-seed puts its rows
  into `4cc_08_bins.cpk` (decision 2026-10-05), but in multi-CPK mode that CPK is the
  compiler's own bins output (`settings.md` `bins_cpk_name`), and the bins walk starts below
  the output CPK (`pipeline.md` "Bins accumulation"). So the DLC builder's next multi-CPK
  compile reads the three player tables from below the seed and replaces `4cc_08_bins.cpk`
  without the seed rows of every player no export compiles. Resolved (2026-10-05,
  maintainer): in multi-CPK mode the player tables' walk starts at the bins CPK itself
  (`pipeline.md` "Bins accumulation").
- answered (2026-10-05, below) — motions on Fox: `settings.toml`'s `[appearance.motion]`
  (hunching, arm movement, kick motions, celebrations, dribbling) are player-record fields
  outside the appearance block (PES 20/21 bits 96 to 332), so neither the `PlayerAppearance.bin`
  row (the block's bytes) nor the Fox patch (names only) carries them: on Fox a compiled
  export's motions reach nobody. Resolved (2026-10-05, maintainer): the Fox patch carries
  every compiled player's motions (`operations.md` "Aesthetics patch"). Untested: that the game
  uses a stripped record's motions (expected: they are outside the appearance block).
  Where a save's motions come from (2026-10-05): a PES 19+ EDIT generated by the game holds
  no player records (the vanilla PES 21 one, `.tmp/4_0/apptest/vanilla21/`); the cup's players are
  injected by `pes-db-generator`'s `player_edit.py`, every record one shared base body
  (`Player_Edit_Base_20.bin`: every motion stored 0, i.e. 1, the motion edit flag set). The
  Winter Cup 2023 save's 4,829 players with a motion other than 1 were edited after that.
- answered (maintainer, 2026-10-08, after Test 4c: a pre-Fox game never applies a
  `GloveList.bin` row, so a full pre-Fox DLC needs the savefile patch anyway, and writing
  `PlayerAppearance.bin` and `BootsList.bin` beside it would split one player's look between
  the CPK and the save for nothing; pre-Fox writes no player tables, the aesthetics stay in
  the savefile patch whole; `pipeline.md` output table, decision entry) — pre-Fox has the
  player tables too (found 2026-10-05):
  `BootsList.bin` and `GloveList.bin` in the base data of PES 15, 16 and 17 (`dt33`, and PES
  17's `dt00_win`), and in the cup's own `4cc_02_misc.cpk` on PES 15 and 17, with a row for
  every cup player id 70101-89223, all 1; `PlayerAppearance.bin` (13,242 rows of 60 bytes, as
  on Fox) and `RefereeAppearance.bin` (35 rows) in PES 17's `dt00_win`; no `PlayerAppearance.bin`
  on PES 15 or 16 (`.tmp/4_0/apptest/prefox_bins/`). Unknown: whether a pre-Fox game reads them
  for a save player (a Test 1 on PES 17 would tell), and what the cup's all-1 rows are for. If
  it does, pre-Fox could carry boots, gloves and (PES 17) appearance in the CPK as Fox does.
  The cup's all-1 rows are placeholders (maintainer). The cup's PES 17 `4cc_01_db.cpk` also
  ships a `PlayerAppearance.bin` with a row per save player. Test 4
  (`.tmp/4_0/apptest/out_test4/GUIDE.txt`, 2026-10-05) repeats Test 1 on PES 17: /b/ stripped
  (appearance id at record byte 116, body 128 to 184; the same 72-byte block as PES 21's at
  240), the three tables as loose files in Sider's livecpk root. Results (run 2026-10-06):
  - **Appearance:** PES 17 reads `PlayerAppearance.bin` for a -1 record, per player (look A,
    and look B on no. 3).
  - **Boots:** it reads `BootsList.bin`. Every /b/ player wore k0563 (brown boots), the
    control DESU included. That's unlike PES 21's control, but DESU's save boots are 0, so
    either the row beats a kept id on PES 17, or a save boots field of 0 defers to the row.
    The placeholder all-1 rows fit either reading only if k0001 looks like the default boots.
  - **Gloves:** the goalkeeper's g0556 row showed bare hands. That's inconclusive: the 4cc
    exe's gloves patch, as on PES 21, or the table is not read.
  - **Control's look:** inconclusive. DESU has a custom head and the same skin colour as look
    A.

  So pre-Fox (PES 17 at least) could carry appearance and boots in the CPK as Fox does. What
  is still needed: the boots reading above, and gloves on a stock exe.
  Test 4b (run 2026-10-08 by the lead through the in-game harness, `.tmp/4_0/ingame/game.py`;
  files by `apptest build` with `APPTEST_CONTROL2`, installed by `.tmp/4_0/apptest/test4b.py`):
  Test 4 plus a second control, no. 4 ITS A TRAP, his appearance id kept, his save boots set
  to k0571 (grey sneakers) and his BootsList row k0563 (tan boots, everyone else's). In Edit
  mode (Edit Pony > Appearance) he wore k0571 while nos. 1 to 3 wore k0563, DESU (save boots
  0, row k0563) included: **a set save boots id wins; only a save boots id of 0 defers to
  the row.** Gloves, on the 4cc exe and on the stock one (`.tmp/4_0/apptest/exe_swap17.py`; the
  stock exe is the oldest `.old` by mtime, 2016-10-20): no. 1's hands were bare in Edit mode
  and in the match walkout both times, with a GloveList row g0012 and his DB appearance row's
  PlayerGloves bit at 0. Still inconclusive: the row did not apply, but the bit may be what
  turns gloves off; the next run sets that bit on his row (the tool's `gloveflags` finds it).
  Test 4c (2026-10-08; 4b's files rebuilt by `apptest build` with `APPTEST_GLOVES_FLAG=70701`,
  one bit changed, no. 1's row body byte 10 bit 0, checked against 4b's tables; installed by
  `.tmp/4_0/apptest/test4c.py`): no. 1's hands were bare again, in Edit mode on the 4cc exe
  (the lead, `.tmp/4_0/ingame/t4d_p1_full.png`) and in a match (the maintainer). **The
  PlayerGloves bit does not turn a GloveList.bin row on; on PES 17 the row never applied, with
  the bit at 0 or 1, on either exe.** Pre-Fox gloves stay in the savefile (the full savefile
  aesthetics patch, Test 1's design answers). Also settled by the maintainer: Edit mode shows
  a player as a match does, goalkeepers included, so an in-game look test needs no match;
  only an outfielder's 2nd+ kit needs one.
- answered — referee collars, `referee_collar_<ID>` or `collar_<ID>`
  (Fox answered below: `collar_<ID>`; pre-Fox answered 2026-10-08 at the end: both, the drawn
  one being `referee_collar_<ID>`): the pre-Fox base
  `uniform_config.xml` loads a referee's collar from `nocloth/referee_collar.model` (type
  `referee_shirt`), Red's pre-Fox referee template ships `referee_collar_026.model`, the cup's
  `4cc_04_fpc.cpk` ships `referee_collar_105.model` beside `collar_105.model`, and PES 21's
  `dt35_g4` holds `referee_collar_*.fmdl`. If referees load `referee_collar_<ID>`, the marker
  (4.27) is `referee_collar_077`, teams and referees do not share collar IDs, and the 77
  reservation against teams (`kit_collar_reserved`, the conflict) guards nothing. Also: a
  collar is drawn with the kit texture, so the marker texture becomes every referee kit's
  main texture (maintainer, 2026-10-05; referee models are full-body and ignore it), and the
  bundled marker models need collar materials. Settle before 4.27. Test 5
  (`.tmp/4_0/apptest/out_test5/GUIDE.txt`, 2026-10-05) gives every referee config collar 77 and
  ships different models as `referee_collar_077` and `collar_077`, on PES 17 and PES 21. Found
  building it: on the installed PES 21 the referee configs are entries of `4cc_08_bins.cpk`'s
  `UniformParameter.bin` (`referee_ACL_1..4.bin`, `referee_DEF_1..5.bin`), and the refs CPK
  holds none, against `blue_port.md`'s "never entries of the bins CPK's
  `UniformParameter.bin`" (Red's Fox template ships them loose); PES 17's are loose files in
  `4cc_35_referees.cpk`, as planned. Results (run 2026-10-06):
  - **PES 21:** the referee wore `collar_077` (the hair). Fox referees load `collar_<ID>`,
    so on Fox the plan stands: the marker is `collar_077`, and the 77 reservation against
    teams guards a real clash.
  - **PES 21 configs:** the collar change made in the `UniformParameter.bin` entries took
    effect, so the game reads referee configs from those entries. Settled 2026-10-07
    (maintainer, from earlier tests): the game needs the loose configs and reads the values
    from the entries, for referees and team kits alike; `blue_port.md` now writes both on
    Fox, step 4.27b. 4.27a went ahead on the plan as written
    (2026-10-07, autonomous mode): the marker's model, texture and reservation hold either way,
    and only where the collar-77 configs go would change. Found then: the compiler's bundled
    `UniformParameter18.bin` and `19.bin` hold `referee_ACL_*`, `referee_DEF_*` (and on 18
    `referee_CL_*`) entries with collar 105, carried forward like the installed bin's, so a
    Fox game that prefers entries keeps showing collar 105 under the marker. The test to settle
    it: a refs CPK holding the collar-77 loose configs and `collar_077.fmdl`, with the bins
    CPK's referee entries left at 105; the marker shows, or not.
  - **PES 17:** neither model showed (with Sider). That's inconclusive. Candidate causes:
    - the loose referee configs were not read from the livecpk root;
    - the two models fail to draw as collars. They are Grigori's face and hair, skinned to
      head bones, with `uni_collar` as a material name, which the stock referee collar
      (`referee_collar_026.model`) does not use: its only material is `uni_shirts`.

    Settled 2026-10-08 (the lead, in-game harness, `.tmp/4_0/apptest/test5b.py` and
    `test5c.py`): **PES 17 referees draw `referee_collar_<ID>`, and need `collar_<ID>` to
    exist.** Found first: the cup's `referee_collar_026.model` (`4cc_35_referees.cpk`, Red's
    template file) has no faces, so the referee's usual look has no collar model at all, and
    Test 5's "neither showed" compared with nothing visible; a loose file in Sider's livecpk
    root changed nothing either way (runs A to E there were void for the same reason). Through
    the test CPK (`4cc_90_test.cpk`, the DpFileList's last entry; the base game's
    `referee_collar_026.model` from `dt35_win.cpk`, 3,204 faces, as the visible model):
    - G, that model as `referee_collar_026`, configs untouched: a V collar drawn (the CPK is
      read; the referee loads `referee_collar_026` by name).
    - H, configs 77 and the same file: the referee absent from 48 walkout frames; A, configs
      77 and the visible model as `referee_collar_077` alone: absent too.
    - B, configs 77 and the visible model as `collar_077` alone: the referee present, no
      collar.
    - I, configs 77, the visible model as `referee_collar_077` and the FPC's empty collar as
      `collar_077`: the referee present with the V collar.
    So the loose referee configs in a CPK are read, the referee's drawn collar is
    `referee_collar_<ID>`, and a missing `collar_<ID>` drops him from the cutscene. For 4.27's
    pre-Fox marker: the marker model goes out as `referee_collar_077.model` with an empty
    `collar_077.model` beside it (as the cup's `4cc_04_fpc.cpk` pairs `collar_105` with
    `referee_collar_105`), and the 77 reservation against teams keeps its point, since a
    team's `collar_077` would be the one the referee finds. "Absent" is an inference from the
    referee not appearing in any frame of a walkout where runs B, G and I show him.
- open, needs the maintainer — FoxDen per-kit models: `docs/QUESTIONS.md` "FoxDen per-kit
  models"; `kit_variant_model_fox` and `model_format.md` "Kit-dependent assets" change with
  the answer.
- open — the unknown bits across versions: `settings_toml.md` names them by record bit
  position per version and never converts them, but the ingame-face run's layout is the same
  on every version (`model.md` "The ingame-face run"), Team TOML prefix-copies it between
  versions today, and a PES 16 export may be compiled for PES 21, so per-version tables drop
  hair and sliders a conversion could keep. Settle before Phase 5 removes the hex: tables
  named by position in the run rather than the record, or converted by the run's offset
  (review S5.A1 item 8).
- open — the base-copy ID on pre-Fox: `model.md` makes it compiler-owned ("derived from what
  was written"), but no plan says what the compiler derives, and the aesthetics patch, the
  only savefile write path, has no key for it (Team TOML has `base_copy_id`). Settle with
  Phase 5's patch: the rule and its key (review S5.A1 item 10).
- open — the three Fox player tables with no installed copy: a compile with no PES folder or
  DpFileList builds the bins on bundled bases, but there are bases only for `TeamColor`,
  `UniColor` and `UniformParameter` (`pipeline.md` "Templates and fallback bins"). For
  `PlayerAppearance`, `BootsList` and `GloveList`: a bundled base, the compiled rows alone, or
  no table? Settle before 4.22 (review S5.A1 item 12).
- open — shared IDs and retained rows: a team's shared boots/gloves IDs are assigned in
  alphabetical folder order from the export being compiled (`player_folders.md` "Assigns IDs
  automatically"), while a player the export does not compile keeps his installed row
  (`pipeline.md` "Bins accumulation"). A `Midcup` export holding shared folder B but not A
  gives B the ID A had in the full CPK, so A's wearers, their rows untouched, now wear B.
  The same holds for pre-Fox save IDs. Settle before 4.22: a midcup keeps the full export's
  assignment (from where?), or must carry every shared folder its team uses (review S5.A2
  item 1).
- open — generation from a stripped save: `settings_toml.md` has generation skip a player
  at -1, so an old-format export migrated against the cup's stripped Fox save gets no
  `settings.toml` for him and his next compile writes all-default rows over the look the
  tables hold. Reading his installed table rows (the bins walk) instead would keep it.
  Settle with Phase 5's generation (review S5.10 item 3).
- open — where the aesthetics patch goes: "beside every CPK a compile publishes" names no
  place in multi-CPK mode (teams parts, the bins CPK, the refs CPK, placeholder parts) or
  in sideload mode (no CPK, yet the savefile step applies "the patch just written"), and
  `patch_written` names one path. The fixed name breaks the pairing in single-CPK mode too:
  two `--no-deploy` runs with different `cpk_name`s keep both CPKs in `output/` but only the
  second patch (review S5.A3 item 1). And the patch is written after the CPK is promoted,
  outside the deployment transaction, which names only the CPKs and the savefile
  (`pipeline.md` "Run-result semantics"), with no finding for a failed patch write, which
  leaves a new CPK beside the old patch (S5.A3 item 4). Settle with Phase 5's patch: one file
  per run or per CPK, its name, its place, and publication with the CPK (review S5.10 item 5).
- open — the Export upgrader's `fpc_on` detection (`export_upgrader.md` step 8, "settings
  match the enable preset"): the cup's FPC players mostly ride their bodies in per-player
  boots IDs, so their boots field holds that ID, not 55, and Test 1's save had every /a/
  player FPC while `is_fpc_player` found none (`.tmp/4_0/apptest/results.txt`, run 2). Matching
  the strip fields alone would also mark a dressed player with long sleeves, tucked shirt and
  short socks. Settle the fields compared before Phase 6 (review S5.11 item 2).
- open — strip-and-seed and a base-copied face: the `PlayerAppearance.bin` row carries no
  base-copy id (`model.md`, "The Fox database row"), which holds a face-transplant
  relationship (`operations.md` "Save-to-save operations"), so a stripped player whose
  record copied another player's face may change face, against "stripping changes nobody's
  look". Untested (Test 1 did not cover it). Measure before strip-and-seed is built; then
  carry the relationship or report the affected players (review S5.A3 item 3).
- open — an unwritable `livecpk\`: sideload mode writes it "with the same access path as
  deployment to `download/`, elevation included" (`pipeline.md` "Output modes"), but no
  plan says what a sideload run does when it cannot (a tree has no `output/` to degrade
  to). Settle with the sideload step (review S6.1b item 21).
- resolved (2026-10-05, maintainer) — the DpFileList outside a compile that deploys: sideload
  and test mode report `dpfilelist_missing` as a Warning and build on the bundled bases
  (`messages.md`, TC-OUT-18).
- resolved (2026-10-02) — the VPS mutation half's memory peak reached 7.07 GiB of the 8 GiB cap
  on `team_compiler` (3.y whole-crate run, 2 build jobs); no build was killed, and a killed one
  now fails the run. The maintainer keeps 8 GiB. (The 37 OOM kills of 2026-10-02 04:00 were the
  3.z run under the old 6 GiB cap, the reason it was raised; none since.)

- open — the deep pass reads everything `compile` reads again (4.7): each checked file is
  read once by the pass and once by its task, and a solid `.7z` is decompressed twice (its
  check, then its tasks; three times before 4.7f for an export holding a metadata file).
  For a folder the second read comes from the system's file cache; for a `.7z` it is a whole
  extra decompression per export, about 2 s for a 0.6 to 1 GB export (4.7f's timing): about
  2 s of a 5.2 to 6.8 s compile. The buffer is not kept from the check to the tasks because
  planning needs every export validated first, and holding every `.7z` until then is the
  residency the budget forbids (`libs/pipeline.md` "What a solid `.7z` is charged"). Not
  designed: keeping the buffers that fit the budget and letting go of the rest. Cup exports
  are nearly always compiled from folders, so this waits for the maintainer's word
  (`docs/QUESTIONS.md` "Solid `.7z` exports over the memory budget").
- open — a face diff is engine-specific (maintainer, 2026-10-03): a diff authored for one
  engine misplaces the face on the other; the compiler passes a diff through for whatever
  target it compiles, so a diff is the author's responsibility, as with Red. The
  investigation waits for the maintainer's go (`docs/QUESTIONS.md` "Face diffs are
  engine-specific").
- open — u16 face indices cap a reassembled split mesh (found at 2.20f review): `fmdl::Mesh`
  and the IR (`ir.md` "IR struct") store faces as `[u16; 3]`, so `fmdl::ops::split::decode`
  refuses (loud `VertexMismatch`) an add-on file whose components together reference more
  than 65536 vertices, and vertex-limit splitting of an IR mesh can only move loose vertices.
  The 2.20h conversion census found 114 real files over it. Since 2.20h such a group stays
  split rather than erroring (`conversion.md` split rules), so nothing refuses them now. The
  face type is still Phase 7's decision (glTF brings u32 indices).
- open — texture names and case on Fox (found at 4.4b2): stems now compare case-folded when a
  player's sources share one, but `processing/model.rs` still points a model's texture path at the
  player's folder only when the referenced name matches a stem exactly (since 4.3), so a model
  naming `Shirt.dds` beside `shirt.dds` keeps its original directory; the written `.ftex` keeps
  one source's spelling while each model keeps its own. Whether the game's CPK lookup is
  case-insensitive is unmeasured; measure it (or fold both sides) before 4.20 opens every export.
  Also: per-stem conflict decisions are independent, so with two conflicts in one folder a
  package can be dropped over a stem whose winner is a package already dropped for another stem
  (rare; needs two differing stems across three packages).
- open — `model_convert` converge questions (2.20): (1) Fox decal shaders (`translucent`,
  `3ddc`, `eyeocclusion`) infer `Shaded` *exact* through the `3ddf` rule, so the highneck
  fixture converts to an opaque `Basic_C`, where the 19to16 converter wrote `Overlay` with alpha
  blending; the format plan deliberately gives decals no family — decide whether the rule should
  at least mark them approximate. (2) Hand split: a face inside both hands' selections goes to
  both gloves; unused materials/textures are not pruned from the split parts. (3) `retarget`
  returns `ConvertError::Validation` after mutating the IR (no rollback). (4) `.model` tangent `w`
  is 1.0 on import (plan bullet); the bitangent-derived handedness is untested in game.
  (5) A `Metal` material without an `Environment` texture exports `Basic_CNSR` with no
  `EnvironmentMap` sampler; the format plan assigns the template-cubemap fallback to the
  compiler's texture step (Phase 4), which must add the sampler, not just the file. (6) Konami
  FMDLs carry a 64-byte-per-bone `bone_matrices` block the IR does not; the add-on writes it
  empty and its files work in game, so the export drops it with a finding; what the block holds
  and whether it is derivable from `Bone.matrix` is unmeasured.
- open — `pes_savefile` PES 21 team record: (1) the reference's colour bits read zero for 203 of
  220 teams of the 4cc save, and the PES 17 save's colours for the same teams appear at no 6-bit
  offset anywhere in the 21 records, so the save most likely carries none (kit configs supply
  colours on 19+); the reference's 21 colour positions are unconfirmed, not disproven. Check what
  4ccEditor displays for that save before the Save editor shows colours on 21. (2) The 21 record
  holds a kit-slot block at +48 (`00 40 af 00 | 01 40 af 00 | 80 40 af 00`: numbers 0, 1, 0x80
  with team 701 x 0x40) that the reference reads only on PES 17 (at +28); the 18-21 tables have no
  `KitSlot*` rows. Measure 18/19 and add the rows when the Save editor needs kit bindings.
- open — `pes_savefile` unmodeled player-record bits that real saves set (2.20i,
  `.tmp/2_20/save_census/src/bin/gaps.rs` over every census save; retained on write, so nothing is
  lost): PES 18 bits 252-254 (3 bits, set in about a third of records; most likely `Star`,
  3 bits on 19/20, absent from the 18 schema); PES 16 bits 222 and 362 (437 records each).
  Model them when the Save editor needs them.
- open — `pes_savefile` player id versus appearance id (2.20i reviewer): the appearance
  block's player id (+116 on 17-19, +240 on 20/21) is unmodeled, so changing
  `PlayerEntry.id` leaves it stale. Nothing changes an id today. Step 4.0 settled what the field
  does (-1 = the player's appearance comes from the database tables), so the model gains it with
  the Phase 5 work that strips and preserves it (decision entry "aesthetics travel in the
  database tables").
- open — FoxDen's PES 2021 gloves trampoline discards `GloveList.bin` (step 4.0, runs 4-7):
  `build_botch_21` in `Tools_4cc/FoxDen/scripts/01_patches.lua` (and the patched exe it
  reproduces) loads the gloves id from `[rsi+0x2C]` for outfield players and goalkeepers,
  overwriting the value the original code chose in `[rsp+0x70]`, which for a player whose save
  appearance id is -1 is his `GloveList.bin` row (the stock exe showed it; the patched exe showed
  bare hands). Request for FoxDen's maintainer: keep the original value when the player has no
  save appearance record, for outfield players too (p1 routes them through the goalkeeper path),
  then an in-game retest with a `GloveList.bin` row for an outfield and a goalkeeper player.
  The Team compiler's gloves output on Fox depends on it.
- open — `pes_savefile::ops::fpc::is_fpc_player` finds no FPC player in a real cup save (step
  4.0 survey of the 2023 Winter Cup save): FPC teams there give each player a blank boots and
  gloves id of his own (`/a/`: 126/126 ... 148/148) rather than 55/11, which its doc comment
  already calls indistinguishable. The Save editor cannot use it to tell FPC teams apart; a hide
  strip (long sleeves, tucked, short socks) plus boots no other player wears marked them in the
  survey (`.tmp/4_0/apptest`, `fpc` subcommand).
- **Texport write is unverified in-game** (2.17h): `Texport::new` synthesizes 18-21 files from
  measured templates and `to_bytes` rewrites read files; both round-trip byte-identical, but no
  generated file has been imported by the game yet (`verification.md` "Texport write": manual,
  per version; a `new` file and an edited round-tripped one). PES 15/16/20 texports have no
  fixture at all (offsets/key index are the reference's; PES 20's size is derived).
- open — FoxDen's LiveCPK gaps that sideload mode depends on (forwarded by the maintainer to
  FoxDen's maintainer, 2026-10-03; source `Tools_4cc/FoxDen/src/livecpk.c`, `patterns.c`):
  (1) the call sites are found on PES 2018 only, 2019 and 2021 still to locate; (2) PES 2020 is
  not recognised (proxy only); (3) a served path is limited to about 99 characters (redirect plus
  name in the read request's 0x80-byte field; a face texture's folder alone is 59), longer names
  silently fall back to the CPK; (4) only boot-time shader loads are verified in game, model,
  texture and kit-bin loads under `livecpk\` are not (Sider's buffered-copy and CPK-size hooks
  were not ported). Phase 4's sideload acceptance waits on them; recheck FoxDen when itemizing.
- open — a run that does not end cleanly leaves its staging (3.z): closing the window while
  the GUI's compile runs ends the process mid-run, and the coordinator's `Cancelled` path does
  not discard the staging either, so `output/.staging/<run>/` can be left beside the previous
  CPK (which stays intact). Fixed with cancellation (`gui.md` "Cancellation", Phase 8): the
  window stops the run and the cancelled path calls `deploy::discard`. The same step adds the
  check between a texture task's textures (4.6's slice (c)); a test cancels a real run partway.
- open — a texture link beside a combined shared folder's texture of the same stem (4.11d):
  a player holding `hair.dds.common` and a link to a shared face whose folder holds
  `hair.dds` compiles with no finding; the shared folder's copy wins, the link is ignored,
  and `hair` is packed twice (the player's home and the team's Common output).
  `texture_stem_conflict` covers the player folder's own files alone. The plan is silent;
  the likely rule is a conflict finding, since collisions are rejected and never silently
  resolved, and it belongs with the stem namespace in `aesthetics_export` or with the
  textures task's conflicts. Rare: it needs a shared folder combined into a player who
  also links one of its stems to `Common/`.
- resolved (2026-10-07: stems fold since `6c2e060`, S3.2, and 4.29's lookup folds too) — texture stems are matched as spelled when a model's path is pointed at its texture
  (4.5, seen again at 4.11c and 4.11d), while validation folds case: a model naming
  `hair.dds` beside `Hair.dds` or `Hair.dds.common` keeps a game path with no finding.
  4.29's existence check is where it would surface; decide there whether the match folds
  case.
- open (2026-10-09, 4.y-fix4) — the deep pass answers "does this link feed the player's own
  package" with an empty hand-weighted set (`deep/mod.rs` `FaceUse::of_player`), because it
  is the pass that finds the hand-weighted models, while planning and validation answer
  with the real set. For a Fox player whose gloves link combines only through his split
  hands, the linked folder's textures do not count for his `.mtl` paths there, so a
  `mtl_texture_not_found` Warning may be spurious (read, not run); nothing is dropped.
  `FaceUse::combined`'s doc comment ("Empty for a team player ...") was already untrue on
  Fox. Fix: find the hand-weighted models before the face use is built, or build the face
  use after them.

---

## Log

Dated, newest last, three lines at most: what changed and what it means for the next session.
No rationale (→ plan), no decisions (→ `DECISIONS.md`).

- **2026-09-07** — Worklog created alongside `AGENTS.md` and `DECISIONS.md`. Planning phase
  near completion; no code yet, no git repository yet.
- **2026-09-10** — Methodology pass: code style reframed and extended, logging and lint decided,
  toolchain pin, `GLOSSARY.md` added, acceptance/converge/rewrite steps added to every phase. Still
  no code and no repository; next is Phase 1.
- **2026-09-10** — First duck review (`gpt-astra-high`) on the methodology docs: 3.5 min
  wall-clock, 7 concerns, 6 accepted and fixed (step-local verification, converge scope vs.
  deferred plan items, Phase 8 acceptance, converge-script modes and scan scope, reactive-review
  exemption); 1 is a real plan contradiction awaiting a user decision (`reqwest` vs. "no async
  runtime" — see `DECISIONS.md`). Nothing it raised had been caught by any prior pass.
- **2026-09-11** — Sessions move to Fusion (Claude lead + SWE-2 sidekick). Sidekick probe: a
  `TeamName` crate implemented cold from `CONTRIBUTING.md` + `team_compiler/README.md` §"Export display
  name and team name" in 3.5 min wall-clock, red-first, all gates green, style rules followed
  literally, two real plan gaps reported rather than decided (leading separators; Unicode vs.
  ASCII lowercasing — the latter is a genuine open point for `teams_list.tsv` matching, carried to
  Phase 3). Second-opinion checkpoints redrawn for three roles (`AGENTS.md`). Phase 1 starts in
  Fusion from step 1.1, with the lead hand-writing the gate infrastructure only.
- **2026-09-12** - Plans for the Refs arranger (Fox hook lists), aesthetics patch, Team creator
  (+ `libs/team_widgets`) written; license settled (`MIT OR Apache-2.0`, Konami data excluded).
  Repository started: first commit replaces the 2023 placeholder history on `the4chancup/4cc-studio`.
- **2026-09-13** - Phase 1 steps 1.1–1.5: workspace, gates, CI file, `vtree` (sidekick),
  `pes_version` and non-GUI `studio_core` (lead), 30 tests, all gates green locally. Two plan
  gaps decided by the user (`PesVersion` home, `unicode-normalization`); font licenses allowed.
  Parallel lead/sidekick work in one tree tripped once on a half-declared module; rule added to
  `AGENTS.md`.
- **2026-09-13** - Phase 1 converge and rewrite done (7 reviewer concerns, all fixed; 35 tests,
  gates green). Converge order flipped to own-audit-first in `AGENTS.md`. Phase 1 closes once the
  first push produces the two CI runs step 1.2 asks for; then Phase 2 starts at 2.1 `wezlib`.
- **2026-09-13** - Second reviewer pass on Phase 1 (5 concerns, all fixed; 38 tests). Commit
  subjects are Conventional Commits from here on; `just` stays on PowerShell for Windows.
- **2026-09-13** - Phase 1 closed: CI green on the first push, red on the planted warning,
  reverted. Phase 2 starts at 2.1 `wezlib`.
- **2026-09-13** - 2.1 `wezlib`, 2.2 `cpk` done (writer parity with pes-file-tools proven);
  fixtures for `fpk`/`ftex` extracted with provenance READMEs; `weszlib` renamed `wezlib`.
- **2026-09-13** - 2.3 `fpk`, 2.4 `ftex`, 2.8 `uniparam`, 2.11 `fpc`, 2.12 `teams_list`, 2.13
  `kit_config` done; 97 tests workspace-wide. The `resources/*.wikitext` pages, committed empty
  in the first commit, were restored from the IDE's local history. Two methodology additions in
  `AGENTS.md`: no legacy tool names in code; the sidekick's report must list every error it hit
  and its fix. `dds_convert` (CPU) briefed.
- **2026-09-13** - 2.5 `dds_convert` (CPU), 2.6 `fmdl` and 2.7 `pes_model` (all but
  `ops::merge`) done; reviews A and B closed (2.5c, 2.12b, 2.13b); 262 tests workspace-wide. Two
  censuses recorded in `libs/README.md` (2610 `.model`, 945 `.mtl`). Safeguards for scripts and sidekick
  trees added to `AGENTS.md` after a truncated source file.
- **2026-09-13** - `resources/prefox_model_format.md` written for the plugin author from the
  census. Merge home decided by the user: native `pes_model::ops::merge`; six plan passages and
  `AGENTS.md` updated, decision logged. Next: implement 2.7d-4, then 2.9 `fox2`.
- **2026-09-13** - 2.7d-4 `pes_model::ops::merge` (bone tolerance measured over 2606 files),
  2.9 `fox2` (87-file census, four fixtures, reviewer pass), 2.10 `archives` (`sevenz-rust2`,
  `zip`), 2.14 `color_tools` extraction (134-kit harness), 2.15 `elevation` done; 329 tests
  workspace-wide, 18 crates on the wasm32 gate. Next: 2.16 `model_convert`.
- **2026-09-14** - 2.16c `model_convert::formats::fmdl` + `loss.rs` done (one rework: the
  vertex-loop encoder reorders Konami vertices, so round trips compare against the decoded input
  re-encoded); `to_fox`/`to_prefox` family defaults only without a native table; dummy maps only
  for family-derived materials. Bone-order census over 2606 `.model` files for 2.16d: 44 list a
  child before its present render parent. Next: 2.16d `formats/pes_model.rs`.
- **2026-09-14** - 2.16d `model_convert::formats::pes_model` done; two censuses behind it
  (`.model` bone order over 2606 files, `.mtl` entry order over 941 files) and the `.model`
  matrix layout checked against PES17 `body.skl` on the fixtures. Retargeting plan section
  sharpened (standard/custom rule, fold mechanics, no-op guarantee); fold literals measured for
  its tests. Next: 2.16f `skeletons/retarget.rs`.
- **2026-09-14** - 2.16 `model_convert` complete: 2.16f retargeting, 2.16g hand split (Blender
  5.2.1 reference fixture), 2.16h routing with the legacy 19to16 reference, 2.16i cross-family
  review (six of seven concerns fixed). 93 crate tests, 422 workspace-wide, wasm32 gate on 19
  crates. Converge questions in "Issues". Next: 2.17 `pes_savefile`.
- **2026-09-14** - 2.17a `pes_savefile::container` done: nine real saves decrypted in the census
  (15/16/17/18/19/21; no 20), slice fixtures plus zlib payloads committed, discovery table
  corrected (PES 18 is flat). Field tables for 2.17b derived by symbolic interpretation of the
  reference read walks and checked against every payload (method in the plan; the
  script survives as `scripts/derive_savefile_schema.py`). Next: 2.17b schema + codec + `PlayerEntry`.
- **2026-09-14** - 2.17b–d done: generated schema tables (`scripts/derive_savefile_schema.py`),
  codec, player/team models, `EditFile`, `display_name`, discovery; 37 crate tests; all nine real
  saves lossless through `EditFile`. Stopped before 2.17e for two user decisions: the
  `settings.toml` key table (export text format) and the ingame-face run, which no legacy tool
  decodes beyond eleven feature types (converters) and a few colour bits, so `PlayerSettings`
  cannot cover "every appearance field" without an opaque-bytes model of that run.
- **2026-09-15** - Method review against "Why Software Factories Fail" (humanlayer). `AGENTS.md`: briefs sized for one review (about 500 lines, slices otherwise); red-run evidence per new test in the brief's closing section; the review sweep split into an honesty sweep and a design sweep; converge gains a design-health pass. Phase 3 tracer-bullet question recorded as step 3.x.
- **2026-09-15** - User decided: Phase 3 opens with a tracer bullet (step 3.3; `core/development_plan.md` "Phase 3" first bullet; decision entry). Early `studio` shell stays an open question (3.y).
- **2026-09-15** - User decided: Phase 3 also closes with a minimal `studio` shell (step 3.z; `core/development_plan.md` "Phase 3" last bullet, "Phase 8" note; decision entry).
- **2026-09-19** - Reviewed 4ccEditor's `Tactics` (`4a95b7c`) and `Autumn_2026_AATF` (`cf61542`)
  branches against the plans. Schemas 15-20 match the new tactics decoders offset for offset;
  `pes_savefile/README.md` gained the Texport `.ted` 18-21 crypto and per-version offsets (for 2.17h),
  `.4cct` layout and the canonical advanced-instruction mapping. `save_editor.md` AATF section
  moved to the Autumn 26 ruleset (bronze tier, specials, two upstream errata recorded for the
  user to report upstream) and, by user decision, to one self-contained Rhai rules file (decision entry). No code.
- **2026-09-19** - Mutation testing adopted after a probe on three closed crates (decision
  entry): `just mutants <crate>` at converge, `just mutants-diff` at every diff review. Review
  round C (2.19a): the probe's survivors plus a workspace read-through fixed in six commits;
  the whole-crate runs on cpk/ftex/dds_convert left 138 survivors for 2.20. One brief premise
  was wrong and the gates caught it: "SKL parents precede children" is false for the game's
  `body.skl`, so the new parent check compares against the bone count. `AGENTS.md`: briefs name
  the `karpathy-guidelines` skill and the consumer crates' tests; `drop(` joins the honesty
  sweep. Next: 2.17e.
- **2026-09-19** - Housekeeping: the session scratch heap is gone and the scripts it held that
  the plans and fixture READMEs cite live in `scripts/provenance/`; the seven plans over a
  thousand lines are folders (`plans/README.md` "How these documents evolve", decision entry);
  the spec stays in `docs/plans/` rather than moving into crates. Pointer-only change, checked
  over every tracked Markdown file. Next: 2.17e.
- **2026-09-19** - User decided the two 2.17e questions: the `settings.toml` key table (one key
  per line, range comment on the same line) and option A for the ingame-face run (opaque bytes
  with typed accessors, "every decoded field"); ranges verified against the reference editor's
  lists and the converters' caps. 2.17e split into three slices; e-1 briefed next.
- **2026-09-19** - 2.17e done in four commits (`912237c`, `a43d1ad`, `97102ea`, `4084541` plus
  the review rework): the ingame-face run, `PlayerSettings` with the key table and the TOML
  half, `ops::fpc`; checkpoint (b) review closed. Next: 2.17f `convert.rs` (the 46/50-byte run
  padding rule is its first decision).
- **2026-09-20** - 2.17f done in three commits (`f3ad268` plan, `1001ba9` playstyle lists +
  caps, `bd2ec4c` `convert.rs` plus the review rework): canonical `PlayStyle` with the reference
  editor's twelve arrays as golden data, `face_type_cap`, `convert_player` as a rewrite into a
  target-version template (prefix copy of the ingame-face run, `min(len)` bytes). Checkpoint (b)
  closed, one plan sharpening (what is and is not a `ConvertNote`). Next: 2.17g
  `ops/{transplant,fingerprint,compare}`.
- **2026-09-20** - 2.17g opened: the reference scripts read (transplant = block bytes 4..68,
  compare = fifteen fields plus a masked SHA-256 prefix; the reference editor's comparator =
  roster-slot pairing over the gameplay fields); the run's tail bytes measured zero on every
  16+ fixture player, so the transplant copies the block whole (decision entry). Plan section
  written with the API blocks, two golden tests written, 2.17g split into two slices.
- **2026-09-20** - 2.17g done (`03459eb` plan + goldens, `6226a6e` transplant/fingerprint,
  `74285bd` compare, plus the review rework): the transplant copies the appearance block whole
  (tail bytes measured zero), the comparator walks the schema's stored fields and tags rows by
  scope, `FaceRun` fires only for undecoded bits. Checkpoint (b) closed. Next: 2.17h
  `interchange/{team_toml,legacy,texport}`.
- **2026-09-20** - PES 20 verified on two real saves (a 4cc invitational's day-0 save and the
  game-generated one): key, header, discovery layout, section offsets and the 20 tables all hold;
  both rewrite byte-identical with their own salt; `compare` between them finds exactly the 414
  named players. Day-0 save is now the `pes20` fixture, in `FIXTURES` (every fixture-wide test
  runs on it; two non-vacuity floors carry its measured thinness). PES 20 open issue closed.
- **2026-09-20** - pes-db-generator planned into the suite: `db_generator.md` (tool, Phase 19,
  scheduled by need), `libs/pesdb` (the Konami table layouts; Ball/Stadium bins move there),
  `pes_savefile::ops::populate` (the 19+ player-section fill). Decision entry. Next: 2.17h.
- **2026-09-21** - 2.17h done (`6122b09` plan, `3575b19` fixtures + goldens, `8385437` texport,
  `2d9365c` instruction + Team TOML team half, `ceb9b6f` player half, `1ae6e16` legacy,
  `28e2eab` review rework): the interchange formats measured on the real files (texport 18-21
  is the save's records concatenated; PES 20/21 store a 17th instruction), Team TOML as the one
  import path with schema-gated notes and stored ranges, `.4ccs`/`.4cct` readers. Crate 113 → 181
  tests; four decision entries. A flaky `file` test fixed on the way: PES 15's one-byte seed made
  `assert_ne!(saved, original)` fail once in 256 saves; the test now saves a PES 16 fixture.
  Next: 2.18 `python_bindings`.
- **2026-09-21** - 2.18 done: the `pes_models_native` wheel (codec `read`/`write` only, the Blender
  accessors wait for the extension's hot-path step), `just bindings`, the CI job deferred from 1.2.
  Mutation runs capped like the builds (`.cargo/mutants.toml`: two mutant processes at 7 build
  jobs / 7 test threads each). Next: 2.19.
- **2026-09-21** - Review rounds are now bounded by accept rate (`AGENTS.md`), and the rule applied
  retroactively to 2.17h: a second round found seven more real concerns (2.17h-6), the largest a
  15-17 texport that wrote a reordered squad and reopened short; a third found three (2.17h-7),
  all the same class (a value `apply` accepted and the codec refused at write), and stopped the
  loop. Next: 2.19, then 2.20 with the bounded loop per crate.
- **2026-09-21** - 2.19 done: gates, deps-check and bindings green locally at `36848c9` and in CI
  on both platforms (658 tests over 40 crate binaries). User decisions: 2.5b GPU BC7 deferred to
  Phase 4 (decision entry); 2.20 runs per crate with the bounded reviewer loop, the tiny leaves
  batched into one reviewer round, `pes_savefile::interchange`'s reviewer half counted as done by
  2.17h's three rounds. Next: 2.20, tiny leaves first.
- **2026-09-23** - 2.20c `cpk` done. Method finding: the reviewer's first round returned five,
  and its reasoning trace (read by the user) showed it aiming for "3-5 strong concerns" because
  the brief said "do not pad"; a second round run as an experiment returned three verified
  concerns, all accepted (row-count allocation with a zero row length, contradicting a lead
  ruling; a `Tvers` long enough to overwrite the first file; NUL paths colliding in the string
  pool). `AGENTS.md` "Second opinion" now continues on the accept count alone (five or more
  accepted, whatever was returned) and bans "do not pad" from the brief. Next: 2.20d `ftex` +
  `dds_convert`.
- **2026-09-26** - 2.20d `ftex` + `dds_convert` done after six reviewer rounds (7/7/7/7/5/4
  accepted); the loop closed on the accept-rate rule with no ceiling. Method finding: a reviewer
  resumed from the previous round found a strict subset (4 of 7) of what a fresh one found on
  the same surface, so reviewers stay fresh (`AGENTS.md` "Second opinion"). Next: 2.20e `fox2`.
- **2026-09-27** - 2.20e `fox2` done after three reviewer rounds (5/5/2 accepted). The
  maintainer ruled that inputs come from a trusted environment, so concerns that need a crafted
  file are rejected (`AGENTS.md` "Fundamental concepts"); two of round 1's were. Finding: the
  reference is not always the oracle: its 33-64 byte hash leaves a sum unmasked, so for
  non-ASCII text we follow CityHash (the game's hash) and pin it with goldens that assert the
  disagreement. Next: 2.20f `fmdl`.
- **2026-09-27** - 2.20f `fmdl` done after four reviewer rounds (7/5/7/4 accepted). The
  maintainer asked that a lead recommendation be applied and logged rather than asked
  (`AGENTS.md` "When the plan has gaps"). Finding: every round found real defects in paths no
  fixture exercises (split output that did not reload, a hang, panics); `pes_model` is the
  `fmdl` port, so its audit starts from this list. Next: 2.20g `pes_model`.
- **2026-09-27** - Method review after 2.20f (`AGENTS.md`): whole-crate mutation runs at the
  start and close of a crate's converge only, `mutants-diff` for the rework rounds between;
  reviewer briefs rank concerns by reachability; briefs tell the sidekick to insert tests
  inside the test module. Maintainer decided the executable is `4cc-studio` (decision entry).
  `fmdl`: the combine reorder gate's `>`/`>=` survivor was a missing test, now written.
- **2026-09-27** - Maintainer decisions: `archives` refuses password-protected archives at open
  (a zip with any encrypted entry, a 7z with an encrypted header or an AES-coded block), so the
  live check reports them at once; maintenance mode is a Phase 16 deliverable with its outline in
  `core/development_plan.md`, not per-file ADRs (decision entries).
- **2026-09-27** - `just mutants <crate>` splits the run with the maintainer's VPS when
  `STUDIO_MUTANTS_REMOTE` is set (`scripts/mutants.py`; `AGENTS.md` "Environment" has the
  benchmark). Verified on `archives`: the split's totals equal a local run's (24 caught, 6
  unviable). Its first whole-crate use is 2.20g's `pes_model` run.
- **2026-09-27** - Maintainer decisions: `fmdl`/`pes_model` `check` flag vertices more than 5000
  units from the origin (Team compiler: `vertex_too_far_from_origin`, folder dropped); code no
  longer names or alludes to the legacy tools (`CONTRIBUTING.md` rule broadened, design sweep
  entry in `AGENTS.md`), swept across 44 files; parity tests keep their tool names.
- **2026-09-28** - Release target set (maintainer): 0.1.0 after Phase 8 (Windows, CPU textures,
  native models, updater check + notice); Phase 7 and GPU BC7 (now 16.x) move after it; no
  Pre-Studio input path. Plan-only change (decision entry). Next: another plan change, then 2.20g.
- **2026-09-28** - `settings.toml` gains top-level stock `boots_id`/`gloves_id` (0 to 100, `""` =
  default: FPC marker or savefile decides); plan-only change across ten documents (decision
  entry), code step 2.17i added. Test 1 files for the appearance-fallback idea are in
  `.tmp/4_0/apptest/out/`, awaiting the maintainer's in-game run.
- **2026-09-28** - Hard gate set (maintainer): step 4.0, the appearance-fallback test, stands at
  the end of Phase 3; no agent goes past it until the maintainer reports the result. Phase 2's
  remaining converge, 2.17i and Phase 3 proceed. Next: 2.20g `pes_model`.
- **2026-09-28** - 2.20g `pes_model` done (one reviewer round, 4/4 accepted). Method finding:
  the Konami-measured reader refused about 1900 of the machine's 6037 community `.model`
  files, cup exports among them, and no fixture, mutation run or reviewer would have found
  it; a census of every real file of the format now belongs in each format crate's converge
  (`.tmp/2_20/model_census/` as the template). Next: 2.20g-fmdl census, then 2.20h.
- **2026-09-28** - 2.20g-fmdl census and 2.20h `model_convert` done. The fmdl census found only
  SKL tails, and every other failure was a correct refusal. The conversion census found about
  one failure in six (weights, split groups, bone order, flags, our own empty containers),
  each traced by measurement before a rule was written. Reviewer rounds 7/3. Next: 2.17i, then
  2.20i `pes_savefile`, opening with a savefile census.
- **2026-09-28** - 2.17i done. 2.20i started: the savefile census found three text-field
  classes that read but do not round-trip (full-length names with no NUL on PES 16/17, and a
  CP1252 byte in a PES 17 name). The session was stopped on the maintainer's request during
  the whole-crate mutation run (partial results in the 2.20i step). Next: decide the text
  rules, fix, rerun `just mutants pes_savefile`.
- **2026-09-28** - Maintainer rulings: the CP1252 save stays refused (4ccEditor cannot load it),
  the 21845-face limit stays. Text fields hold their full length (decision entry). Test 1's
  step-by-step guide is `.tmp/4_0/apptest/out/GUIDE.txt`. Next: the 2.20i text fix.
- **2026-09-28** - 2.20i text fix landed (`cd3a1d1`); census clean. Whole-crate run done
  (24 survivors, `.tmp/2_20/mutants_2_20i_whole/missed.txt`), the crate's last (maintainer). Lead
  audit in `.tmp/2_20/audit_2_20i.md` (F1-F7). Next: slice A = survivors + F1-F4, F6, F7; slice B =
  F5's pure moves; each checked with `mutants-diff`; then the reviewer.
- **2026-09-28** - `just mutants` fixed: the remote half runs detached and survives a dropped
  link or a killed controller (`just mutants-collect`), and shells without
  `STUDIO_MUTANTS_REMOTE` read it from the registry.
- **2026-09-28** - Maintainer check of the full-field shirt name (`EDIT00000000 prespoon`,
  PES 16): 4ccEditor shows `MEAT ON THE BON`, and the save crashes PES 16 after the start
  screen, cause unknown (the maintainer doubts the shirt name). `shirt_name_from` keeps its
  free byte. Next: 2.20i slice A (`.tmp/2_20/brief_2_20i_a.md`).
- **2026-09-29** - 2.20i slice A done (survivor tests, F1-F4, F6, F7; `mutants-diff`: 0
  missed). Correction: the `cd3a1d1` whole-crate run measured 1130 of 1611 mutants, not all
  of them (its VPS half died partway); the maintainer approved one run of the 486 unmeasured.
- **2026-09-29** - 2.20i remainder run: 502 mutants, 4 missed (3 equivalent as written,
  rewritten; 1 test added), `mutants-diff`: 0 missed. Next: slice B.
- **2026-09-29** - 2.20i slice B done: pure moves split `team_toml/team.rs` (tactics, items)
  and `settings_toml/mod.rs` (document), checked by `.tmp/2_20/pure_move_check.py`. Next: the
  reviewer.
- **2026-09-29** - 2.20i done. The reviewer's round (3 of 4 accepted) found PES 18's
  dribbling-arm motion stored in 3 bits where the schema read 2; a census of set-but-unmodeled
  bits found no other such field. Overfull rosters and malformed PES 15 descriptions are
  now refused at write. Next: 2.20j `python_bindings`.
- **2026-09-29** - Memory caps after a mutant (`display_name`, `i *= 1`) OOMed the VPS: the
  remote half is a capped, CPU-idle systemd unit, and Windows test binaries run under a 6 GB
  per-process cap (`scripts/test_runner.py`). Both caught the mutant when re-run under the cap.
  A census of a bit-packed format now also tallies unmodeled set bits (`AGENTS.md`). Next:
  2.20j `python_bindings`.
- **2026-09-29** - 2.20j done: the wheel's smoke test, stale since 2.20g-h's fixtures, passes
  again and runs inside Blender 5.0/5.2; mutants measured by hand (11 of 12 caught); reviewer
  5/1 accepted. The VPS is off limits for mutation runs until the maintainer says otherwise
  (gotchas). Next: 2.21, the Phase 2 plan rewrite.
- **2026-09-29** - 2.21 in progress: Phase 2's development-plan section, `model.md`'s model
  blocks and stale library names rewritten. It found a gap, now 2.20k: savefile field values
  were never compared with 4ccEditor's. Next: 2.20k.
- **2026-09-30** - 2.20k done: 4ccEditor's own `.4ccs` exports from the fixture saves equal the
  codec's read on every carried value, PES 15-21. It found `.4ccs` records ordered by save
  record, not roster; `read_squad` now maps by the target's record order. VPS mutation runs
  allowed again (maintainer). Next: the rest of 2.21.
- **2026-09-30** - `just mutants-diff` splits with the VPS when its estimated local time is at
  least 4 minutes, estimated from each crate's measured seconds per mutant
  (`target/mutants-cost.json`); the 2.20k diff's slower half took 502 s against 717 s local-only.
- **2026-09-30** - Phase 2 closed: 2.21 done. An as-built audit of the Phase 2 plans against
  the code (`.tmp/2_21/audit_2_21_as_built.md`) fixed stale crate trees, deps rows, two code blocks
  and unmarked Phase 8/19 parts. Step list collapsed. Next: 3.1, Phase 3's Acceptance section.
- **2026-09-30** - 3.1 done: Phase 3's Acceptance section, the entry gates ("Validation
  semantics") and the CLI exit codes, after 8 GPT review rounds and 4 sidekick rounds. Reviewers
  are now uncapped (decision entry). Next: 3.2, the acceptance-ID scanner.
- **2026-09-30** - 3.3 done: the tracer compiles /egg/'s Stormworks player face and kit g1 for
  PES 21 and matches Red's output on every row Phase 3 produces (FTEX by decoded content,
  decision entry). Six Phase 2 frictions listed as 3.4. Next: 3.4.
- **2026-09-30** - 3.4 done: `kit_config::KitSlot`, FTEX headers in the parity comparison, the
  plan's stale `fmdl_id_change` and the CPK tool version settled (decision entry). Next: 3.5.
- **2026-09-30** - 3.5 done: the `aesthetics_export` structure pass shapes written into
  `object_model.md` (decision entry). Next: 3.6 slice (a).
- **2026-10-01** - 3.6 (b) rules settled in `object_model.md` "Validation semantics" (allowlist
  per position, OS artifacts, folder-name split, issue scope, eligibility, drop order, kits/
  portraits/logo details; decision entry, census of 225 OS artifacts in real model folders).
  Slice b1 landed (`mutants-diff`: 81, 59 caught, 15 unviable, 7 missed: 4 got tests, 2 went with a
  redundant roster guard, 2 are the dropped-player filter b1 cannot reach and b2's tests cover). Next: b2.
- **2026-10-01** - 3.6 slice b2 landed: allowlist, links, markers, reserved subfolders, stems,
  Fox `fmdl_name_invalid`, the cascade and pass-through. Review rework: one link resolver, shared
  helpers for drops/strictness/mapped slots. `mutants-diff 8b17f4c` (b1+b2): 221, 18 missed, all
  given tests; rerun 198 caught, 23 unviable, 0 missed. Next: b3.
- **2026-10-01** - 3.6 slice b3 landed: kit grammar, duplicates, `all/` inheritance, kit
  allowlist/names/markers/icon, portraits, logo grammar and roles, root allowlist and notes
  (`kit_config` dependency). Review rework: one stem-conflict helper (b2's two copies included),
  shared icon/marker/name checks, an invalid kit head gets no other finding (decision entry).
  `mutants-diff 3f838a4`: 114, 15 missed (14 given tests, 1 equivalent removed by restructuring);
  rerun 111, 97 caught, 14 unviable, 0 missed. Next: the reviewer on all of (b).
- **2026-10-01** - 3.6 cross-family review, GPT round 1: 7 concerns, 4 accepted (fold panics on
  real names → `vtree::fold_name`; flattening panic → insertion-built tree, no root findings on an
  undecided root; texture `.common` links and `Common/` join the stem check; TC-STR-09 uncited
  until 3.8), 3 rejected; the GPT loop ends (rulings `.tmp/3_6/review_rulings_3_6.md`; plan + decision
  entry). `mutants-diff c936e0e`: 31, 2 missed, both given tests; rerun 26 caught, 5 unviable, 0 missed. Next: the sidekick
  review loop.
- **2026-10-01** - 3.6 sidekick review S1: 7 concerns, 3 accepted (a kept disallowed `.common`
  file stays in the player's files; no logo findings on an undecided root; allowlist wording for
  `all/` and `Portraits/`), 4 rejected; the loop ends, and with it the 3.6 review.
  `mutants-diff 78a3190`: 5, 4 caught, 1 unviable. 3.6 done. Next: 3.7, plan section first.
- **2026-10-01** - 3.7 started: `libs/pipeline.md` written (permit, cancellation, oversized
  priority, the solid-7z charge, thread count, `CpkStem`; decision entry), then the crate's
  budget, thread count and `CpkStem` (11 tests, each group red against a stub; the oversized-
  priority test red against a budget without `oversized_waiting`). The memory cap is a share of
  the memory available at run start, read per OS (`windows` feature approved by the maintainer;
  decision entry); its `memory.rs` is not written yet. Not yet run: `mutants-diff`, the
  cross-family review (queued, see "Handover").
- **2026-10-01** - 3.7 implemented: `pipeline::memory_cap` (Windows `GlobalMemoryStatusEx`,
  Linux `/proc/meminfo` through a `mem_available` parse tested on every platform). `mutants-diff
  af9da18`: 12 missed, 7 given tests (exact-cap request is ordinary, OS read and cap above a
  floor, the 4 GiB fallback, the parse); the 5 left are the Linux and other-platform
  `available_memory` arms, compiled out on Windows (the Linux arm is checked by
  `the_os_reports_available_memory` on Linux CI; the other arm is a constant `None`). 17 tests.
  Next: 3.8; the 3.7 review is queued.
- **2026-10-01** - 3.8 shapes settled (decision entry; plan edits in `core/architecture.md`,
  `distribution.md`, `team_compiler/settings.md`, `messages.md`, `pipeline.md`) and sliced
  a-d. 3.8a landed: `cli_run` returns `Result<u8, CliError>`, `AppPaths`, CLI data location
  (never writes), the binary's console printer. Red: the unknown-tool test failed `left: 1
  right: 2` against a code-1 refusal; the rest is new code. Implemented by a Sonnet subagent
  (handover note), reviewed by the lead. Next: 3.8b.
- **2026-10-01** - 3.8a re-checked at the maintainer's request: `load_settings` moved from a
  `String` error and `current_exe().ok()` to `anyhow` with context; missing why-comments added
  (`BaseDirs`, the printer fallback, the printer-failure exit code, now 3). 3.8b landed: the
  tool skeleton and preflight; red: all 10 binary tests failed `unexpected argument
  'team-compiler' found` before the subcommand existed. Subagents now Opus (maintainer). Next: 3.8c.
- **2026-10-01** - 3.8c landed: `check` over folder sources, the catalog, discovery and
  routing, findings as events. Red: 15 of 22 in-process tests failed against 3.8b's stub (e.g.
  TC-SRC-05 `left: []`, TC-CLI-01 `refused with 3: check is not built yet`). Rulings on the
  subagent's open points: an unreadable root aborts (3); sources are listed before routing,
  so an unreadable one never counts as a refs duplicate; `export_extract_failed` carries
  `path` and `error`; the refs summary precedes the exports. Found: `mutants-diff` skips
  untracked files (fixed in the detour below); cargo-mutants copies the gitignored `.tmp/` into its build
  copies, which filled the disk (3.7's two diff runs died of it). Next: the mutants/`.tmp`
  detour (maintainer), then 3.8d.
- **2026-10-01** - Detour: neither 3.7 diff run had finished (the second reported 11 of 67
  mutants before the disk filled), so its numbers above had no complete run behind them.
  Re-run over `af9da18..5618240` (`pipeline` unchanged since): 67, 51 caught, 5 unviable, 6
  timeouts (mutated `MemoryBudget::acquire` wait conditions that block forever), 5 missed, the
  compiled-out Linux and other-platform `available_memory` arms the entry names; run on the
  VPS (Linux, 4c7c424), the Linux arm's 3 are caught, leaving only the other-platform 2,
  which no tested OS compiles. Every other
  Phase 3 run had completed (3.3's and 3.4's survivors triaged in their steps). Fixed:
  `mutants.toml` sets `gitignore = true` (the copies leave out `.tmp/`, checked in the run's
  `debug.log`); `mutants_diff.py` diffs against the working tree written as a tree object
  (`mutants.working_tree`, shared with the remote snapshot), so untracked files are measured
  without staging. `AGENTS.md`: archived runs keep only their summaries.
- **2026-10-01** - Detour done. Cleanup (maintainer-approved): the 10 stale `%TEMP%` copies,
  archived runs cut to their summaries, `bench/`, census and probe build outputs; 63 GB freed,
  `.tmp/` 17 GB to 0.8 GB. `teams_list.txt` marked `-text` in `.gitattributes`: CRLF by its
  crate's contract (a test compares the shipped list with the crate's own write), so the
  remote snapshot's fresh `git add` had stored it LF. Next: 3.8d.
- **2026-10-01** - 3.8d landed: archive sources in `check` (`archives::folders()`, one open per
  source, the 7z read charged per `libs/pipeline.md`). Red: TC-SRC-01/02/06/07/08/09's check
  tests all reported `archive sources arrive in step 3.8d` before the reader change. Lead's
  review: a directory entry naming the root (`./`) skipped instead of refusing the archive
  (decision entry); the brief's "cite every scenario" overrode 3.8's rule, so the tags of
  the four compile-half scenarios were removed (44 proven). `mutants-diff 4c7c424`: 28, 21
  caught, 7 unviable, 0 missed. `team_compiler` now depends on `archives` (planned, in-tree).
  `mutants.toml` notes the `available_memory` platform arms; 3.w (mutants.toml audit) and
  4.0a (PES12 re-study, stash `pes12`) added at the maintainer's request. Next: 3.9.
- 2026-10-02: 3.9a landed after one rework round. The lead's review of the first diff found
  game-facing loosenings (any `.bin`/`.fclo` packed into `face.fpk`, the `fcl_hair.skl` pairing
  check dropped, any image read as DDS), unreadable files reported as `folder_pack_failed`, kit
  scopes rebuilt as `Kits/<name>`, an empty `.staging/` left behind, two struct lifetimes and a
  three-noun `check` module; all fixed (decision entry "3.9a review"). The task's files are now
  read by the coordinator, which 3.9c's pool needs. The slice ran ~600 lines with 14 audit items
  folded in, over budget: later slices carry no unrelated fixes. `mutants-diff 6c254c3`: 146,
  109 caught, 36 unviable, 1 missed: the `.7z` permit choice (3.9c's fixture).
- 2026-10-02: 3.9b landed first time. A design pass (Plan subagent) before the brief found
  the scope's "files a lenient setting keeps do not count" unimplementable (the validated
  export does not mark them); the plan now lists what counts (decision entry "3.9b"). The brief
  settled every rule, and the diff followed it; its one reported gap (a `pass_through`-kept
  `.common` link) is already handled by the lib, which takes such a link off `files`. `mutants-diff 2c1f31e`: 68, 63 caught,
  5 unviable, 0 missed.
- 2026-10-02: 3.9b2 landed first time, tests only: all 17 scenarios passed on the code as built.
  The brief's one wrong expectation was the lead's (a kit with no `config.toml` also reports
  `kit_config_generated`). With `pass_through`, a folder that `shared_link_duplicate` drops still
  shows its `link_target_missing` lines as `[Keep]`: each finding carries its own disposition,
  and the grid's cell (Phase 8) derives the folder's outcome from all of them. No mutation run
  (no `src/` change).
- 2026-10-02: 3.9c landed first time. The sidekick found two brief gaps, both accepted:
  `crossbeam-channel` was only a dev-dependency of `team_compiler`, and draining the channel
  after a write error still left the writer's waiting batches holding permits an oversized
  `acquire` could wait on forever (fix: `submit` clears them; red-first test). A write error
  drops the messages of batches not yet reported; the run aborts on that error anyway.
  `mutants-diff 1306cad`: 16, 10 caught, 5 unviable, 1 timeout (the `.7z` permit choice
  `==` → `!=` deadlocks under the over-cap test, as it should: 3.9a's survivor is gone).
- 2026-10-02: 3.9d landed first time, the first slice by a Fable sidekick (maintainer's trial).
  Its three reported brief errors were all right (no catalog `F` kind existed; TC-OUT-03's
  chain names the face's texture, the first duplicate the writer meets, not `face.fpk`;
  `promote` reuses `discard` whole), and it caught one the brief missed: the writer's
  `CpkWriter` would still hold the staged file open when a failure removed the folder.
  `mutants-diff cb6f0cf`: 12, 10 caught, 2 unviable, 0 missed.
- 2026-10-02: 3.9e landed first time (Fable), closing 3.9 at 73 of 75. A kit folder holding only
  `config.toml` is a placeholder kit too, so one older planning test gained a `kit_placeholder`
  line. Lead fixes: the help paragraph's "It skips" read as the kit folder after the new
  sentence; a second CPK-reading helper merged into `cpk_entries`. `mutants-diff 9072f79`: 11,
  10 caught, 1 unviable, 0 missed.
- 2026-10-02: 3.z landed after one tests-only rework round (Fable): its first diff passed review
  as written, and `mutants-diff` left 11 survivors in the window code. Eight are now proven
  headless (`egui::Context::run_ui`, AccessKit bounds for clicks, `eframe::Frame::_new_kittest`);
  `run_gui`/`run_gui_mode` open a native window and are excluded (`mutants.toml`, TC-GUI-01
  proves them); the lead added the test that a poll never blocks on a run in flight. Final
  local `mutants-diff 44c284f`: 38, 27 caught, 7 unviable, 4 timeouts (each a mutant that
  never ends the run), 0 missed. `egui` raised to 0.36.2 with `eframe`. Manual TC-GUI-01/02
  pending the maintainer.
- 2026-10-02: mutation tooling (maintainer's report and idea): `mutants-diff`'s estimate now
  adds the measured baseline and averages every outcome (it said 80 s for a 270 s run; studio
  and studio_core had been counted as one crate, `src`); local runs start at below-normal
  priority with half the CPUs per job; a build killed for memory is listed and fails the run
  instead of passing as unviable (12 remote mutants of 3.z, eframe's tree past the 6 GiB cap).
- **2026-10-02** - 3.9f: the maintainer's TC-GUI-01/02 run passed; with no exports folder it
  printed a raw OS error, so the relative exports folder is now created before a run, a missing
  one named any other way is refused in a sentence naming the path and the setting (exit 2),
  and an empty one reports `no_exports_found`. Red runs: TC-CLI-08 exit `3` against `2`;
  TC-CLI-09 and its `check` half "the folder was created"; the `cli.rs` unit tests against an
  unconditional `Ok(root)`. Fable: landed first time.
- **2026-10-02** - 3.w: `.cargo/mutants.toml` audited entry by entry (row 3.w); five entries
  were wrong or too broad, the config now removes 132 mutants instead of 150, and three
  `model_convert` tests kill the overlapping ORs the old entry hid. Red runs (mutation applied
  by hand): `to_fox.rs:148` `left: 0 right: 32`; `pes_model/export.rs:129` `left: [] right:
  ["no_shadow_cast"]`, `:134` `left: [] right: ["invisible"]`. Fable: landed first time and
  disproved the lead's `quantize_weights` counterexample with a 6-million-input probe.
- **2026-10-02** - 3.y converge, the reviewer loop aside (row 3.y). Rejected sweep findings: the
  headless egui harness's second copy (the third, in Phase 8, extracts it: Issues), the two
  `SmallMetadata` lookups (the parse stage has raw keys, validation only canonical ones), the
  two `Disposition` enums (libs do not depend on `studio_core`; `messages.rs` is the seam),
  `PLACEHOLDER_KIT.to_vec()` and the kit config copies (owned input, small), "legacy" for
  `refs.txt` (the format's history, as `messages.md` says it), `ctx` (the plan's trait
  signature), the plan-named or plan-deferred `pub` items, `Tool`'s `Default` beside `new`
  (clippy's `new_without_default`). Red runs (mutation by hand): `Settings::load` guard
  `matches!(…Io(error) if error.kind() != NotFound)` failed; `gui_run` default `unwrap_err()
  on Ok`; `version_change_blocker` `left: Some("xyzzy") right: None`; `ToolContext::common`
  `left: 0 right: 4`; `exe_dir` exit `Some(0)` against `Some(2)`; `install_cli_logger` `left:
  Off`; `log_level` arm `left: Trace right: Info`. `mutants-diff b6d898c`: 58, 49 caught, 9
  unviable. Fable: landed first time; a name it could not use as briefed (`position` shadows
  the function) renamed `file_position` and reported.
- **2026-10-02** — 3.x: Phase 3 closed, its cross-family reviews queued. The plan's Phase 3
  sections describe what exists (`development_plan.md` "Phase 3" with its done Verification:
  1402 tests, 77 of 77 scenarios); the step list collapsed. Next: 4.0a, 4.0b, then stop.
- **2026-10-02** — 4.0a and 4.0b reported. PES 12: pes12tools reads the old export layout, has no
  license and reshapes daily; keep the stub stashed. Sider: LiveCPK matches the planned Sider mode;
  a launcher over the user's own Sider is recommended. Both await the maintainer's decisions.
- **2026-10-02** — 4.0c: `Engine` is matched exhaustively everywhere (Fable, landed first time,
  0 lead fixes). Sideloader decisions recorded in the lead's state; study part 2 running.
- **2026-10-02** — 4.0b closed: sideload mode and the `overrides/` folder renamed in the plans,
  a plain Launch PES button planned; 4.0d renames the code. Test 1 (step 4.0) installed for the
  maintainer's in-game check, backups beside the save and `DpFileList.bin`.
- **2026-10-03** — 4.0e: sideloading goes through FoxDen's LiveCPK into the game folder's
  `livecpk\`; no Studio sideloader, the untracked stub deleted, PES 15/16 refused. FoxDen's gaps
  (2019/2021 sites, 2020, path length, in-match loads) are under "Issues". Still blocked on 4.0.
- **2026-10-03** — CI's `deps-check` red since 3.z (`a691220`): `eframe` → `arboard` brought
  `clipboard-win` and `error-code` under `BSL-1.0`, and Phase 3's close never ran the recipe.
  `BSL-1.0` allowed; compatible licenses join without asking, and `deps-check` runs right after
  any `Cargo.lock` addition (decision entry). `just deps-check` green locally.
- **2026-10-03** — `(maintainer, <date>)` notes removed from the plans and `AGENTS.md`; the four
  with no decision entry got one, and `AGENTS.md` keeps them out (decision entry).
- **2026-10-03** — Step 4.0 done: seven in-game runs on PES 2021 (the first, on the June save,
  failed on a corrupted `SYSTEM00000000`; the maintainer moved to the 2023 Winter Cup save and
  DLC). Database tables are read for players whose save appearance id is -1; gloves need a
  FoxDen change ("Issues"). "Target versions" added to `development_plan.md` (Fox first,
  game-behavior changes through FoxDen) and the aesthetics design recorded (decision entries).
  Next: the plan rewrite those entries imply, then Phase 4's itemization.
- **2026-10-03** — 4.1: Phase 4 itemized (4.2-4.28 plus 4.y, converge, rewrite) and its
  Acceptance section written (126 scenarios); 27 plan gaps found, the lead-decidable ones ruled
  (decision entry), the rest under "Phase 4 open questions". A VGL26 corpus of 53 old-layout
  exports (`C:/Data/4cc/Lab/Gud`, 15 GB) is the converge census's input once Phase 6 upgrades it.
- **2026-10-03** — Phase 4 pre-step plan text (decision entry "Phase 4 pre-step measurements"):
  portraits pass DDS through, the stock collar sets per version, logo sizes, the `colors.txt`
  grammar, referee paths, the blank face folder and the DpFileList layout, measured on the PES
  2021/2017 installs and Red. The sidekick's three contradictions answered (trailing icon kept,
  `4cc_90_test` on PES 21 asked, cross-engine collars at 4.9).
- **2026-10-03** — 4.2: DDS portraits compile on Fox targets, passed through byte for byte
  (`player_` prefix on PES 18); non-DDS and two-source slots stay refused until 4.6/4.7. Fable
  landed it first time under the new brief shape (Devin's 3.3 model, contradictions invited).
- **2026-10-03** — 4.3: per-team ID blocks and each player's own boots/gloves compile on Fox;
  the tracer now matches all 17 of Red's files. The textures-last shape the lead briefed was
  wrong (a failed texture left dangling packages); the sidekick's folder-group commit replaced it.
- **2026-10-03** — 4.4a: shared Boots/Gloves folders linked plainly compile once under the team's
  shared IDs (644 up for `/co/`), with their textures beside their own package; more than 17 is
  `boots_id_pool_exhausted`/`gloves_id_pool_exhausted`, reported by `check` too. The remote
  mutation half peaked at 7.00 GiB of its 8G cap (40 mutants).
- **2026-10-03** — 4.4b1: boots/gloves parts resolving to one name merge on Fox, and a boots or
  gloves link beside a local model of its kind combines the shared folder into the player's
  exclusive package (`link_combined`, `fmdl_merged`, `skl_merge_conflict`,
  `merge_material_conflict`). The tracer's own files supply real conflicts: its `glove_l.fmdl`
  and `boots.fmdl` disagree on bone `sk_forearm_l`, its `fcl_hair.fmdl` on material `shirt`.
- **2026-10-03** — 4.4b2, closing 4.4: a face link makes the shared face folder part of the
  player's face, face parts merge like boots parts, and a texture stem two of a player's sources
  hold is one entry when the bytes agree, else `shared_texture_conflict` (lower package dropped)
  or `merged_texture_conflict` (folder dropped). Remote mutation half peaked at 7.14 GiB of 8G.
- **2026-10-03** — 4.5a: a Fox face folder no longer needs `face_diff.bin`, `fcl_hair_sim.fclo`
  or a hair skeleton (Red's templates and PES 21's `body.skl` injected), an unsuffixed model is a
  `fcl_hair` part (`fmdl_fcl_hair_fallback`), and a skeleton beside a slotless face model is
  `skl_no_slot`; `check` reports both findings.
- **2026-10-03** — 4.5b: a player folder's `face/`, `boots/`, `gloves/` and `common/` subfolders
  compile on Fox, each forcing its category (Red's referee layout, kept as legacy support).
  TC-MOD-04 re-scoped: two `skin.dds` in one player's subfolders are `texture_stem_conflict`, so
  only a combined shared folder can reach `shared_texture_conflict`.
- **2026-10-03** — 4.5c: a player's `.common` link to an FMDL bakes the Common model into its Fox
  package as a part (its Common skeleton with it), and `Common/`'s `.dds`/`.ftex` are converted
  once per export into the team's Common output, which a Common part's paths name. Each part's
  texture paths are now rewritten before the merge. Texture `.common` links found with no step;
  filed under 4.11.
- **2026-10-03** — 4.6a: every player, shared, Common and kit texture in any accepted image
  format compiles on Fox through `dds_convert` (BC7 on PES 19-21, BC3/BC1 on PES 18, missing
  mips generated, `_nrm` stems as normal maps), with a per-run conversion cache for runs of at
  most two exports. The BC7 encoder now builds optimized in dev (56 s to 9 s for one test).
- **2026-10-03** — 4.6b: a texture that cannot convert names its finding (`texture_too_small`,
  `texture_not_pow2`, `texture_type_mismatch`, `texture_codec_unsupported`) and drops its folder,
  its kit, or only itself in `Common/` and for a portrait; a Fox `kit_mask` is reported
  (`kit_texture_not_used`); portraits come in any format, encoded to BC3 by
  `dds_convert::encode_dds`.
- **2026-10-03** — 4.6 closed: slice (c), the cancellation check between a texture task's
  textures, moved to Phase 8's cancellation, which brings the Cancel that triggers it; the
  conversions in flight are already bounded, one texture at a time per worker.
- **2026-10-03** — maintainer rulings recorded in the plans (eight decision entries): format
  findings drop by severity; a missing texture is an Error with the installed CPKs searched
  (4.29); `base64` for `face_diff.xml`; `colors.txt` one color per line; marker names (4.30);
  the Fox referee marker as a reserved collar (4.27 rewritten); one official DpFileList. Step
  4.31 (pre-Fox parity reference) is the lead's. Lead on Fable 5.1, sidekick on Opus 5.5.
- **2026-10-03** — more rulings recorded: collar 77 reserved for the referee marker on both
  engines, `kit_collar_reserved` for a team kit using it (survey: 4,635 kit configs, 77 unused);
  the texture lookup searches only the CPKs before the one compiled; the official DpFileList's
  slot run is still to be authored (open question).
- **2026-10-03** — 4.30: marker names (`fpc_on`, `fpc_off`, `icon_<N>`), the first slice by the
  Opus 5.5 sidekick, landed first time. Also recorded: a `Collars/` file named for a reserved
  collar (105, 77) is `collar_id_conflict`, not `collar_id_invalid` (decision entry); the kit
  layout pair made with PES Master's two kit creators settles the socks and leaves the shorts
  open ("Phase 4 open questions"); `fpc_toggle.md`'s precedence table no longer says a
  default boots/gloves key with no marker preserves the installed ID (it is 0, as
  `settings_toml.md` has said since 4.0).
- **2026-10-03** — 4.5d: a Fox face folder's `face_diff.xml` is decoded into `face_diff.bin`,
  and every face diff is checked (`face_diff_invalid`, `xml_dif_conflict`); 4.5 is closed.
  Maintainer's answers recorded: the far-vertex rule stands (the census's 89 parked placeholder
  models are what it exists for); the slot stem is `teams`, and the official DpFileList is
  drafted in `resources/templates/DpFileList.txt`; the referee marker's models are named and
  it stays on the ground by static painting, so 4.27 waits on nothing. Measured while the
  external drive was connected: every version's stock collar set but PES 20's (decision
  entry). New issue: a face diff is engine-specific.
- **2026-10-03** — A claim checked on the installs at the maintainer's request: collar 105 is
  missing from the stock PES 15 game, but the cup's PES 15 has FPC all the same (its own FPC
  CPK supplies the collar; 301 of 360 kit configs use the four FPC values). `libs/fpc.md` and a
  correcting decision entry say so; the code follows at 4.9. Two open questions added: PES 18's
  FPC values, and whether the games load collars beyond their stock set.
- **2026-10-03** — 4.7a: the deep pass exists end to end. `check` and `compile` read every
  `.fmdl` of the sanitized export, a solid `.7z`'s included, and a vertex over 5000 units out
  drops its folder (`vertex_too_far_from_origin`), with `pass_through` on too; a dropped shared
  folder or `Common/` model takes the players linking it. The export crate re-derives the
  sanitized export from the findings, so no drop or cascade code exists twice.
- **2026-10-03** — 4.7b: `check` and `compile` now report every model check of the two model
  formats and of `.mtl` files, one line per file and problem with a count. An Error (a mesh
  over a hard limit, a face naming a missing vertex) drops the folder unless `pass_through`
  keeps it; a file that is not a model at all is `model_broken` and always dropped. On the
  VGL26 corpus (2,938 `.fmdl`): 89 files with a far vertex (dropped, as decided at 4.7),
  640 with unnormalized weights (Info), 1 with an empty mesh (Warning), no other Error.
  One rework round: counts of item rules (0 became 1), the catalog test's order check, two
  acceptance IDs. Lead fix: `crates/studio`'s CLI test wrote an empty model, now a real
  one. Also this session: the plan for the texture checks (4.7c, split in two) with its
  decision entry, and 4.8's two bin bases bundled with their measured layout.
- **2026-10-03** — PES 2020 is installed (`C:\Data\Games\PES2020`, and on the external
  drive), with the cup's base DLC. Measured on it: the stock collars are 1-127 and 901-913 (the
  plan had assumed 1-124, what PES 19 and 21 share); its FPC CPK supplies the empty
  `collar_105` and `pants_016` like every other version's; it holds no team kit config and no
  color bins (`4cc_08_bins.cpk` is a placeholder).
- **2026-10-03** — Two maintainer rulings. Color bins (4.8): the compiler writes both whole
  on every run and sets every record's header, which repairs the headerless `TeamColor.bin`
  records installed cups carry (decision entry; `pipeline.md` "Bins accumulation";
  TC-BIN-13). Mutation runs: the VPS memory cap cannot be raised; when a build is killed at
  it the run fails and names `REMOTE_BUILD_JOBS` as the setting to lower (`AGENTS.md`,
  `scripts/mutants.py`). Also: PES 21's `download` folder now holds the last VGL's DLC, and
  its kit-config tally in `libs/fpc.md` is redone on it (300 of 314 with the FPC values).
- **2026-10-03** — The repaired bin headers are reported: `bin_header_repaired` (W), once per
  bin, naming the teams (maintainer; decision entry; `messages.md`, TC-BIN-13).
- **2026-10-03** — 4.7c1: the deep pass checks textures from their headers
  (`dds_convert::probe`), so `check` reports `texture_type_mismatch`, `texture_too_small`,
  `texture_not_pow2` and the new `kit_texture_too_big`, each dropping its folder, kit or
  `Common/` file before planning, and `pass_through` keeps the three size findings' files.
  One tests-only rework round (the size rules' one-sided cases, from 7 mutation survivors).
- **2026-10-03** — 4.7c2: `check` reports a portrait's texture findings and drops that file
  alone (`pass_through` keeps a size finding's file); two differing portraits for one slot
  are `portrait_conflict` and skip the export, identical ones compile as one; a logo that
  does not decode is `logo_file_invalid`. `deep.rs` is now the `deep/` module.
- **2026-10-03** — The official DpFileList is fixed by the maintainer
  (`resources/templates/DpFileList.txt`, 53 entries: stadiums 20-35 with the base CPK at 20,
  `teams` 41-45, `teams2` 51-55, midcups 61-75, `4cc_99_test`), and every compile is to compare
  the installed list with it: `dpfilelist_not_official` (W) when it differs and still lists
  the run's targets, the upgrade offered and never forced. The plan and the steps use the new
  names; the code's default `cpk_name` follows at 4.25a. The list file's edit went into commit
  `90e5bfb` with slice 4.7c2 (the lead's `git add -A`), not into a commit of its own.
- **2026-10-03** — The installs' DpFileLists compared with the official one
  (`.tmp/4_7/dpfl_compare.py`): 29 to 45 entries each, every listed CPK present in `download/`,
  15 to 24 names per install that the official list retires. So the upgrade also writes a
  placeholder for each official entry with no file (decision entry), and two open questions
  are added: a 53-entry list in the game, and an old DLC under the new list. The plan's
  default `refs_cpk_name` is `4cc_18_referees`. PES 21's `download/` on the external drive
  has no `DpFileList.bin` since the VGL DLC was copied in (the old one is in
  `download_old/`).
- **2026-10-03** — The maintainer's answers on the DpFileList: the upgrade renames an old
  DLC's CPKs by stem (best effort; decision entry, TC-DEP-13); a listed CPK with no file
  makes the game reject the whole download folder and run vanilla, so every compile reports
  one (`dpfilelist_cpk_missing`, TC-DEP-14) and the placeholders are confirmed; a list
  lacking the run's CPK stays `dpfilelist_outdated`; the 53-entry list is still to be
  re-tested in the game (about 60 held, years ago).
- **2026-10-03** — 4.7d: `check` reports a broken face diff, a folder giving its face diff
  in both forms, a kit's `config.toml` that does not parse (`kit_config_invalid`, the kit
  dropped) and a player's `settings.toml` that does not (`settings_toml_invalid`, the file
  alone dropped); none is kept by `pass_through`. The deep pass's checks are complete; step
  4.7's timing is left.
- **2026-10-03** — 4.25a: the default `cpk_name` is `4cc_99_test` in the code and its tests.
- **2026-10-03** — 4.7 timed on FNG (991 MB) and DBG (604 MB): `check` 0.55 and 0.82 s as
  a folder, 2.6 and 2.8 s as a solid `.7z`; `compile` 3.1 and 1.7 s, 7.2 and 5.8 s. The
  serial deep pass is up to 47% of a folder compile, so it goes on the worker pool within
  each export (4.7e, decision entry; the maintainer asked for it), and the `.7z`'s three
  decompressions get a step of their own (4.7f).
- **2026-10-03** — The maintainer's answers: cups are compiled from plain folders, so
  folder and `.zip` exports are checked in parallel with each other (4.7e; only `.7z`
  exports in turn, decision entry); a model-less face always takes the bundled face diff,
  the two other files in the VGL26 exports being outdated FPC-dedicated diffs (decision
  entry; 4.12 settles how the leftover file is reported).
- **2026-10-03** — 4.7e: the deep pass runs on the worker pool (folders, their files and
  `Common/` in parallel; folder and `.zip` exports in parallel with each other, `.7z` ones
  in turn; sources routed in parallel). `check` on a folder export went from 0.55 to
  0.32 s (FNG) and 0.82 to 0.34 s (DBG), both in one run from 1.31 to 0.55 s; no finding or
  order changed.
- **2026-10-03** — The 53-entry DpFileList passed the maintainer's in-game test on PES 2015
  and on PES 2021 (one per engine), so the official list's length is taken to hold on every
  version and the open question is closed (decision entry).
- **2026-10-04** — 4.7f, and step 4.7 done: routing only lists each source, and a source's
  check reads its metadata through the handle its deep pass reads from, so a `.7z` holding a
  metadata file is decompressed once for both passes, not twice. `check` on such an export
  went from 4.39 to 2.36 s (FNG) and 4.23 to 2.31 s (DBG); an export with no metadata file
  measures the same as before. This corrects 4.7's timing note: a decompression costs about
  2 s per export, and the timing exports, which hold no metadata file, were decompressed
  twice per compile, not three times. The tasks' decompression stays ("Issues").
- **2026-10-04** — 4.8a: `check` and `compile` read each kit's and the root `colors.txt` and
  warn about every line that does not give one color (`color_entry_invalid`); nothing is
  written to a bin yet. Step 4.8 is in three slices, and a kit's colors will merge into its
  team's `UniColor.bin` record by kit number, where Red rewrites the whole record (decision
  entry; open with the maintainer: whether a kit a team no longer has should keep its entry).
- **2026-10-04** — 4.8b: every CPK `compile` writes carries a whole `TeamColor.bin`, the
  bundled base with each compiled team's root `colors.txt` colors set; a team export without
  the file reports `team_colors_missing` and keeps its colors. The tracer's bin equals Red's
  byte for byte. Kit colors (`UniColor.bin`) are 4.8c.
- **2026-10-04** — 4.8c, and step 4.8 done: every CPK `compile` writes carries a whole
  `UniColor.bin`, the bundled base with each committed kit's two menu colors and icon merged
  into its team's record. A kit without two valid colors in its `colors.txt` gets them from
  its main texture (`kit_colors_derived`), a kit with no main texture gets magenta and black
  (`kit_colors_missing`). The tracer's bin equals Red's byte for byte. Open with the
  maintainer: kits a team no longer has keep their entries ("Phase 4 open questions").
- **2026-10-04** — 4.9a: an export with an `fpc_on` marker compiles. When any player
  folder carries the marker, every supplied kit config lacking the four FPC values gets
  them (`kit_config_fpc_adjusted`); without it configs are emitted as supplied. `check` and
  `compile` warn about a config value the target version clamps
  (`kit_config_version_clamped`). 4.9b (collars) waits on the maintainer.
- **2026-10-04** — 4.10, lead first: the kit layout table is measured and in the plan. Only
  the socks differ between the layouts; they convert in two bands per sock (pre-Fox u 8–168
  ↔ Fox 8–128, 168–448 ↔ 128–376). The shorts are laid out alike: the "shift" the plan
  quoted was the Fox body standing 33 to 44 mm taller, read as a layout change by a
  nearest-point match (decision entry). Golden: `tests/fixtures/kit_layout/`. Found on the
  way: the stock number atlases are a column of digits pre-Fox and a row in Fox ("Phase 4
  open questions").
- **2026-10-04** — 4.10: a kit marked `pre-fox` compiles for a Fox game with its socks
  re-laid to the Fox layout (`kit_layout_converted`); everything else in the texture stays
  as it was, and a DXT kit keeps the blocks the move does not touch. The other direction
  is built and tested at the function; its CLI test waits for pre-Fox targets (4.16).
- **2026-10-04** — 4.11a: `compile` makes the game's three logo PNGs (512, 256, 128) from
  the export's `logo*` image, the smallest from `logo_small*` when there is one; a
  non-square image is fitted, cropped or stretched by its tag. Step 4.11 is split into four
  slices; the plan text and decisions for the next two (notes, kit variants) are in.
  TC-CMN-06 moved to 4.29.
- **2026-10-04** — 4.11b: `compile` gathers the compiled exports' `notes.txt` into
  `output/teamnotes.txt`, one `--- /co/ ---` section per team, once the CPK is in place; a
  compile with no notes removes an older file. TC-TEX-09 (a texture `.common` link) is
  written for 4.11d.
- **2026-10-04** — 4.11c: kit variants on Fox. A texture set (`pants_kit1`, `pants_kit3`)
  gets a copy of its lowest variant for each kit number the team has and the set lacks
  (`kit_variant_missing`); a model's `pants_kitN` path is pointed at where the variants
  go; of per-kit model files only the lowest is compiled (`kit_variant_model_fox`).
  TC-CMN-05 split: its PES 17 half is TC-CMN-07, with 4.14.
- **2026-10-04** — 4.11d: a texture `.common` link (`hair.dds.common`) compiles: the
  player's model names the texture in the team's Common output, where it is packed once.
  Step 4.11 is done. Two open issues recorded: a link beside a combined shared folder's
  texture of the same stem, and stems matched as spelled.
- **2026-10-04** — 4.12: `ingame_face` compiles on Fox (no face folder, the models the face
  would take become the player's boots), every other player folder gets a face folder, blank
  without a face model, and a face file with no face model is `face_file_not_used`.
  TC-MOD-09 reworded: a failed package is left out alone.
- **2026-10-04** — 4.13a: two exports of one team are each skipped with
  `duplicate_aesthetics_export`, by `check` and `compile`, and the other teams compile;
  the CPK is proven independent of the worker count and of skipped exports. 4.13 goes on
  as 4.13b (`overrides/`) and 4.13c (source pinning).
- **2026-10-04** — 4.13b: the files of the data directory's `overrides/` folder go into the
  CPK first and win over an export's entry or a bin at their path (`overrides_active`,
  `duplicate_path`); they are written even when no export compiles.
- **2026-10-04** — 4.13c: an export file that changes while `compile` runs (saved over,
  removed, the archive replaced) aborts the run with `source_changed_during_run`; the
  previous CPK stays. Step 4.13 is done.
- **2026-10-04** — the maintainer answered five Phase 4 open questions: override paths compare
  exactly; a failed package is left out alone as long as the Error is visible (help sentence
  added); number atlases are re-arranged across engines (new step 4.32); one kit config
  template for every version (the census found the template's bytes already the common
  ones, the clamp coming from how they are read); a full export resets a team's kit
  record and a midcup one does not (`Full`/`Midcup` in the export's name, under discussion).
- **2026-10-04** — settled with the maintainer: the kit config template stays and `name.y`
  is PES 21's value on every version (step 4.33); a team export's name carries `Full` or
  `Midcup` as its second word (step 4.34, plan and eight scenarios written: TC-ID-05 to 07,
  TC-KIT-24, TC-BIN-14 to 17; four reworded).
- **2026-10-04** — Review queue, S1 `pipeline` (3.7 (b), 3.y (c)): sidekick S1.1 1 of 5
  accepted; GPT A1 3 of 3 (a finished player group's permit held by the coordinator hung the
  run when the next acquire needed it; the writer released a permit before its bytes;
  `CpkStem`'s rule order untested). Rulings `.tmp/review_queue_S/review_rulings_S1.md`.
- **2026-10-04** — Review queue, S2 `studio_core` + `studio` (3.8 (b), 3.z (b), 3.y (c)):
  sidekick S2.1 0 of 4 (one recorded under "Issues" for Phase 8); GPT A1 4 of 6 (an unwritable
  data folder stopped `compile` at the teams list; a TOML error's snippet broke the one-line
  console format; shared temp folders in settings tests; a `pub` with no consumer). Rulings
  `.tmp/review_queue_S/review_rulings_S2.md`.
- **2026-10-04** — Review queue, S3 `aesthetics_export` (3.y (c), with its Phase 4 parts):
  sidekick S3.1 1 of 9 (an empty doubled layer folder kept as a phantom folder); GPT A1 4 of
  4 (two spellings of one Common model link panicked a compile; skeletons paired case-exactly
  where planning folds; roster findings on an undecided root; a flattened wrapper of empty
  folders kept); sidekick S3.2 2 of 3 (face file names and texture stems matched
  case-exactly). Rulings `.tmp/review_queue_S/review_rulings_S3.md`.
- **2026-10-05** — Review queue, S4 the Team compiler's Phase 3 skeleton (3.8 (b) CLI half,
  3.9f (a), 3.y (c) team_compiler): sidekick S4.1 1 of 4 (the command-line root's refusal said
  no action); GPT A1 5 of 5 (a failed CPK write kept the whole cup processing; a teams list
  written straight onto its path left a truncated file later runs read; `--export ..` named
  `..`; the parity test normalized relocated references unchecked; TC-OUT-06's text stale);
  sidekick S4.2 0; GPT A2 1 of 1 (a `.7z` kept running after the cancel); sidekick S4.3 1 of 1
  (the cancel checked only between sources). From S4.3 the sidekick's role is a `swe-2-high`
  subagent (AGENTS.md "Environment"). Rulings `.tmp/review_queue_S/review_rulings_S4.md`.
- **2026-10-05** — Review queue, S5 the step 4.0 plan rewrite (docs only): sidekick loop S5.1
  to S5.8 (21, 9, 9, 5, 7, 5, 8, 3 accepted); GPT A1 14 of 14 (the Fox rows' stage in the
  diagram, pre-Fox face-XML IDs through the settings order, the patch written per published
  CPK, the unknown-bits example on decoded bits, model.md's copy of the precedence rule);
  sidekick S5.9 4; GPT A2 6 of 6 (generation from a stripped record, the full-length shirt
  name, the face-editor recipe's recompile); sidekick S5.10 to S5.13 (10, 5, 6, 2: one rule
  for every reader of a stripped record, the presets' fields listed only in `libs/fpc.md`);
  GPT A3 4 of 4 (all worklog issues); sidekick S5.14 0 of 2. Thirteen open "Issues" came out
  of it, five for the maintainer. Rulings `.tmp/review_queue_S/review_rulings_S5.md`.
- **2026-10-05** — Review queue, S6 the Team compiler's Phase 4 Acceptance section (docs, plus
  test citations and two test fixes): sidekick loop S6.1 to S6.3 (30, 7, 4 accepted); GPT A1
  22 of 24 (unreachable scenarios withdrawn or rewritten, four new); sidekick S6.4, S6.5 (5,
  1); GPT A2 16 of 16 (pre-Fox scenarios: shared gloves `glove.xml`, Common MTL links, referee
  faces; TC-TEX-12 proven); sidekick S6.6 to S6.12 (14, 10, 9, 6, 8, 11, 4: WHENs and GIVENs
  name what their THEN observes, pre-Fox collars and merges, TC-MOD-42 and TC-ROOT-13 proven,
  referee kit configs pinned as loose files); GPT A3 6 of 8 (TC-TEX-01/10's tests decode the
  payload, not the header); sidekick S6.13 2; GPT A4 4 of 4 (the earlier-only texture lookup);
  sidekick S6.14 3 of 3. Three conventions now head "Acceptance" ("Format and rules"): `Full`
  unless `Midcup`, PES 21 unless a version is named, and a GIVEN's implied least content.
  Five plan questions went to "open first" on their steps (4.9a(b) collar format, 4.14 MTL
  cascade, 4.19 pre-Fox referee boots) or "Issues" (DpFileList outside a deploying compile).
  Acceptance 246 scenarios (2 withdrawn), 148 proven. Rulings `.tmp/review_queue_S/review_rulings_S6.md`.
- **2026-10-05** — the maintainer answered four questions (decision entry): a collar in the
  other engine's format is converted (TC-CMN-09); sideload and test mode report
  `dpfilelist_missing` as a Warning (TC-OUT-18); a `Full` export keeps the rows of team
  players it does not compile; a referee folder is a player folder, so pre-Fox local boots
  ride in its `face.xml` (TC-REF-09). Test 2, a stripped player with no `BootsList`/
  `GloveList` row, is ready for the maintainer (`.tmp/4_0/apptest/out_test2/GUIDE.txt`).
- **2026-10-05** — more answers (decision entry): the PES 16 Common patch is treated as
  existing; `kitN` is FoxDen's on Fox; the stock collar sets need no confirmation (they were
  counted), so 4.9b waits only on where a collar's textures land; a refs export may hold
  shared boots and gloves, each linking referee's `k99XX`/`g99XX` (TC-REF-10). Test 3
  (`.tmp/4_0/apptest/out_test3/`) checks the converted socks and collar 200 in game.
- **2026-10-05** — third set (decision entry): collars are models only, drawn with the kit
  texture (TC-CMN-08..10); multi-CPK keeps the seed rows; the `autopilot` root marker alone
  opts names in, the Fox patch carries motions, and a compile with nothing to patch backs the
  old patch up as `.bak`. Found: pre-Fox player tables, and referee collars named
  `referee_collar_<ID>` (both in "Issues").
- **2026-10-06** — AATF redesigned with the maintainer (decision entry): a ruleset is data
  (`RULESET`) run by a generic Rhai interpreter carried in each rules file, plus an optional
  `custom_checks` hook; the official ruleset uses the same schema; VGL26 (4ccEditor-VGL) is the
  second embedded ruleset and fixture; VGL's suggestions become a third severity. New plans
  `aatf_rules.md` (the format, moved out of `save_editor.md`) and `ruleset_editor.md` (new tool,
  Phase 20). Checked on the way: Rhai is maintained (1.26.0, 2026-08; only its LSP is abandoned);
  upstream fixed the three AATF errata the old section listed (`f5e7b3e`). No code.
- **2026-10-06** — Ruleset editor: Randomize added to the plan (Sensible / Weird / Crazy, seeded,
  every result backed by a legal witness team; decision entry). VGL26 parity now has its save:
  `C:\Data\4cc\Saves\EDIT00000000_VGL26`. No code.
- **2026-10-06** — VTL11 (VTLEditor `ab5f0e2`) checked against the AATF schema with the
  maintainer (decision entry): small general additions, `team_settings` for PES 16 tactics
  checks, the 188 cm positional rule as a custom check; VTL11 is a third test ruleset with parity
  on `C:\Data\4cc\Saves\EDIT00000000_VTL11`. No code.
- **2026-10-07** — 4ccEditor-VGL `vgl27` (`45ce31d`, tag `VGL27.A`) reviewed: `8f86bd3` is
  upstream's Tactics tab and Texport import merged in (covered by step 5.0); `4447996` is the
  VGL27 ruleset, a fourth AATF fixture, adding `registered_position_fielded` (decision entry).
  Its upstream defects were written up for the maintainer to report. No code.
- **2026-10-07** — 2026's official rulesets checked against the schema: Autumn 25 (`7e01541`)
  and Spring 26 (`c92d535`) fit unchanged and join as test rulesets, official parity now running
  each cup save under its own ruleset (decision entry). No code.
- **2026-10-07** — VGL24 (`d4d7b15`) and VGL25 (`2c2d26a`) checked: the optional manlet bonus
  becomes a one-option `choice` with a `choices` suggestion; VGL25 joins as a test ruleset
  (decision entry). No code.
- **2026-10-07** — VGL26 and VGL27 checked against their rules pages: both follow the pages
  (`fielded_at_a` replaces `registered_position_fielded`; `forbidden_shapes` for VGL26's 3-4-3
  ban; exact injury resistance; tactics settings), VGL27's differences added to the bug report
  (decision entry). No code.
- **2026-10-06** — the Clef scan joins the review (maintainer; decision entry):
  `scripts/clef_scan.py`, `just clef-diff [base]` and `just clef <crate>`, rulings in
  `scripts/clef_rulings.md`, a queue for days the free tier's neurons run out. Checked offline
  with a fake client (windowing, an injected bug flagged and located, cache, rulings, the queue
  across a date change); the live check waits for the quota reset. Next for it: one
  `just clef all` pass over the current code, its flags ruled.
- **2026-10-06** — Clef scan: windows measured positional vs function-sized (positional kept,
  decision entry); the report merges overlapping flagged windows; production code now ends at
  the inline test module, not the first `#[cfg(test)]` (5,007 lines were skipped). The
  scan reads several tokens in order with a per-day fallback (`cf96357`).
- **2026-10-06** — in-game Tests 2, 4 and 5 run by the maintainer, results in "Issues":
  - **Test 2:** a stripped Fox player with no `BootsList` row wears boots 0, so 4.22's drop is
    safe for boots; the gloves default is still open.
  - **Test 4:** PES 17 reads `PlayerAppearance.bin` and `BootsList.bin` for -1 records.
  - **Test 5:** PES 21 referees load `collar_<ID>`, and its referee configs are read from
    `UniformParameter.bin`; PES 17 inconclusive.
  - **Test 3:** postponed; Test 3b added for kit 2 (edit mode shows only kit 1). No code.
- **2026-10-07** — implementation resumed (the Devin lead's review rounds paused; the
  maintainer asks for autonomous work, decisions only they can take logged for later):
  - **4.33:** `name.y` is PES 21's value on every version.
  - **Clef full pass (4.c-pass):** 54 flags, 53 rejected and 1 accepted (step 4.c-fix1). The
    scan now leaves out test modules in files of their own. The maintainer took Workers Paid,
    so Clef has no daily limit now.
  - **4.34a:** the coverage tag is read and required.
  - **Maintainer's rulings:** every team needs one player kit and one goalkeeper kit, so a
    `Full` export without them compiles an empty `p1/` or `g1/` (TC-BIN-19, 20; TC-BIN-14
    given a `g1`).
  - **4.21a:** the working bins come from the installed CPKs (the `DpFileList.bin` layout
    measured on four versions' lists); `pes_folder_path`'s `**` is now expanded; the placeholder
    CPK's one `placeholder` file corrected in the plan and glossary.
  - **4.21b:** the `templates/` override folder; an unreadable override now aborts the run
    whatever the resource (decision entry).
  - **4.21c:** a midcup `fpc_on` export patches the FPC values into its team's installed kit
    configs of the kits the game offers that it does not hold; a full export removes its
    team's other configs; a working bin that does not parse stops the run before any export
    is read. Step 4.21 is done.
  - **4.22:** the Fox boots and gloves lists and the player appearance table come from the
    installed CPKs and carry the compiled players' rows; a list no installed CPK holds is not
    written (`player_table_missing`, decision entry).
  - **4.29a:** a Fox model naming a texture in the team's Common output that neither the
    export nor an earlier installed CPK holds is `fmdl_texture_not_found` (Error, package left
    out; Warning when the CPKs cannot be searched). 4.29b next (the texture link).
  - **4.29b:** a texture `.common` link whose target an earlier installed CPK holds is
    satisfied, in `check` and `compile` alike. Step 4.29 is done.
  - **4.23a:** `compile --mode sideload` writes the CPK's entries as loose files that replace
    `livecpk/` once written whole; refused on PES 15/16 and without a PES folder. 4.23b next.
  - **4.23b:** `compile --mode test` writes each export's processed files, models unpacked,
    under `output/test_output/<source>/<folder>/`, the bins under `_bins/`, overrides not
    applied; `test_output/` replaced once written whole. Step 4.23 is done.
  - **4.24:** `compile` installs the CPK into the game's `download/` (a `.partial` copy renamed
    over the old one); when it cannot (no PES folder, the name unlisted, `download/` unwritable,
    the old CPK in use) it says why and leaves the CPK in `output/`, exit 1. A killed run's
    staging is swept by the next run. Every test now runs against a sandbox PES folder.
  - **4.25b:** the official `DpFileList.bin` is a `templates/` resource; a deploying compile
    compares the installed list with it: `dpfilelist_outdated` (E) for an older list lacking
    the run's CPK, `dpfilelist_not_official` and `dpfilelist_cpk_missing` (W), each naming
    `upgrade-dpfl`. Test installs are now upgraded ones (official list, placeholders).
  - **4.25c:** `upgrade-dpfl [--yes]` lists, then makes, the renames of an old DLC's CPKs by
    stem, the placeholders of empty official slots and the official list (old one as `.bak`),
    the list last; it never deletes or overwrites a CPK. Step 4.25 is done.
  - **4.26a:** the `cpk` writer tells the length its file would have with more entries
    (`len_with`), laid out by the code `finish` writes with. 4.26b next.
  - **4.26b:** multi-CPK mode with `--no-deploy`: whole teams first-fit into the official
    list's `teams` slots under `cpk_part_max_size` (a byte count, 3 GiB), placeholders in
    the slots left, overrides and bins in `4cc_08_bins`; `cpk_team_exceeds_cap`,
    `cpk_slots_exhausted` (Fatal) and `cpk_size_over_limit` (single CPK, Warning). A
    deploying multi-CPK run is still refused; 4.26c installs them all or none.
  - **4.26c:** a multi-CPK compile deploys: every CPK of the run judged by the preflight,
    then installed all or none (old CPKs moved aside first, put back on any failure, every
    CPK then promoted to `output/`). Step 4.26 is done; 4.19 (referees) next.
  - **4.19a:** a referee export compiles on PES 18 to 21: each referee folder built once and
    written for every slot `players.txt` gives it (`referee0NN`, `k99NN`, `g99NN`), its
    textures once under `common/999/<folder>/`, a link to `Boots`/`Gloves` made each slot's
    own; a referee export's kit, logo or portrait is still skipped by name. It lands in the
    run's one CPK until 4.19b writes the refs CPK.
  - **4.19b:** the referees go into their own CPK, `refs_cpk_name` (`4cc_18_referees`),
    installed or promoted with the team CPKs, all or none; a compile whose only output is
    the referees writes that CPK alone and leaves the team CPK installed as it was. A
    `refs_cpk_name` naming a team-side CPK is refused.
  - **4.19c:** the referees' CPK also holds the referee kits and `RefereeAppearance.bin` the
    game needs (Red's `refscpk_fox` tree, embedded); a file at `templates/referees_fox/<game
    path>` in the data directory replaces that file. Sideload writes them too. Step 4.19
    (Fox referees) is done.
  - **4.27a:** a `ref_marker.dds` at a refs export's root goes into the referees' CPK on
    Fox with a flat square model written as collar 77, and the referee kits are set to
    wear collar 77; a team kit whose collar or winter collar is 77 is left out with
    `kit_collar_reserved`. Whether PES 21 reads the loose referee kits that carry the
    collar is still to be tested in game (Issues, referee collars).
  - **4.9b1:** `check` reports a `Collars/` file that is no model, a name that gives no
    stock collar of the version a kit can wear (`collar_id_invalid`, the 9xx ones
    included), a claim on 105 or 77 (`collar_id_conflict`), and a collar model's check
    findings as any model's. `compile` still refuses an export that keeps a collar.
  - **4.9b2:** a team's `Collars/collar_<ID>.fmdl` compiles on Fox, and every kit of the
    team wears it, after FPC, kits a midcup does not resend included; the later of two
    exports claiming one collar, or an export's second collar, is `collar_id_conflict`.
    Step 4.9 is done.
  - **4.28:** a running task's decoded textures, parsed models and finished entries are
    charged to the run's memory budget, without waiting, so later tasks wait for them to
    be freed. No member-visible change. 4.y (bounding the conversion cache) moved to
    Phase 8, where the GUI first keeps a cache across runs.
  - **4.32:** a kit's number textures made for the other games (a column of digits for
    PES 2015 to 2017, a row for 2018 to 2021) are re-arranged for the game compiled for,
    each digit moved into the places Konami's own textures keep it, measured on the stock
    textures of both games. The name texture is left as it is.
  - **4.18:** on PES 2018 to 2021, a face model whose vertices are weighted to the hand
    bones has its hands cut off at the wrist into the player's gloves, with their own
    gloves ID and row, and the note `model_hand_split`; a model named as boots or gloves
    is never cut.
  - **4.14a:** a per-kit model's name is read without its kit token, on every game:
    `boots_kit1.fmdl` is boots, not a part of the face's hair, and a shared `Boots/`
    folder may hold per-kit boots, of which kit 1's is compiled on PES 2018 to 2021.
  - **4.14b:** PES 2015 to 2017 compile a player folder's own `.model` files with their
    `.mtl` files, textures and face diff into the player's face CPK, with a generated
    `face.xml` typing each model by its name (boots and gloves models included), and
    the blank face of a player with no model; a model with no `.mtl` is
    `model_material_undefined`. Links, `Common/`, shared folders, kits, collars,
    `ingame_face`, `.fmdl` files and referee exports are still refused on those games.
  - **4.14c1:** on PES 2015 to 2017 a linked `Faces/` folder is copied into the player's
    face (his own file of the same name wins), and a linked `Boots/` or `Gloves/` folder is
    written once under its shared ID as loose files, the boots as `boots.model` and
    `boots.mtl`, the gloves under their own names with a generated `glove.xml`. `Common/`,
    kits, collars, `ingame_face` and referee exports are still refused there.
  - **4.14c2:** on PES 2015 to 2017 the export's `Common/` folder compiles: its models,
    `.mtl` files and textures are written once into the team's Common folder, a
    `.model.common` link puts that model in the player's `face.xml` by its Common path, a
    `.mtl.common` link or a Common `.mtl` found for a linked model is named there too, and a
    texture link points the player's `.mtl` at the Common texture. Kits, collars,
    `ingame_face` and referee exports are still refused there.
  - **4.14d:** on PES 2015 to 2017 `check` and `compile` refuse a folder holding
    `face_edithair.xml` or `hair.xml`, a shared boots or gloves folder's model without its
    suffix (`model_name_invalid`), a texture whose width or height is not a multiple of 4
    (`texture_not_div4`, even with `pass_through`), and a model naming a material its `.mtl`
    lacks (`model_material_undefined`, which `pass_through` may keep); on PES 2015 a
    `uniform` model is written `uniform_sub` (`xml_uniform_pes15`).
  - **4.31:** a pre-Fox parity test compiles a real PES 2017 player (face, boots, gloves)
    and compares the CPK with what the legacy compiler built from the same export. Nothing
    a member would see changes.
  - **4.14e1:** on PES 2015 to 2017 a player folder holding `ingame_face` compiles: every
    model but a glove becomes part of his own boots, merged into one `boots.model` with a
    boots folder his link combines (`model_merged`, `link_combined`; conflicting parts leave
    the boots out with `merge_material_conflict`, `skl_merge_conflict` or
    `model_merge_flags_conflict`), and a shared `Boots/` folder holding several boots models
    is merged into its `boots.model`. Gloves beside the marker are still refused there.
  - **4.14e2:** on PES 2015 to 2017 an `ingame_face` player's gloves compile into his own
    gloves folder, unmerged, each model with its `.mtl` and a `glove.xml`, a gloves folder his
    link combines adding its models (his own file replacing one of the same name); a shared
    gloves folder no longer writes a `.mtl` none of its models uses.
  - **4.14e3:** on PES 2015 to 2017 a face model whose vertices weigh on the hand bones is
    split at the wrists at compile time (`model_hand_split`): the body keeps its `face.xml`
    entry and its hands become `oral_<stem>_glove_l_win32.model` and `_glove_r_` entries
    typed `gloveL`/`gloveR` naming the same `.mtl`; a model named as boots or gloves, one a
    `.common` link brings in, and one under `ingame_face` are not split; a split model whose
    `.mtl` is a Common file is refused (decision entry).
  - **In-game tests run by the lead** (maintainer's authorization of 2026-10-08: PES 2017
    and 2021 on `E:` windowed, saves and game files backed up) through a Python harness,
    `.tmp/4_0/ingame/game.py` (Sider then the game, not elevated; screenshots of the window's
    client area with `mss`; DirectInput keys with `pydirectinput`; `burst.py` tiles frames),
    every game-side change made and undone by a hashed install/revert script: Test 5b/5c
    settled the pre-Fox referee collar names (Issues, referee collars) and Test 4b the
    pre-Fox boots rule (Issues, pre-Fox player tables); gloves stay open there. Every file
    checked back to its baseline by hash after the runs (save, SYSTEM, the exe, the test CPK
    slot, the livecpk root).
  - **4.14e4 done** (TC-CMN-07): on PES 15-17 a per-kit `.model` set is packed whole and
    listed once in the generated `face.xml`, through its lowest variant's entries with the kit
    token spelled `kitN` in `path` and `material` (decision entry: the exe to come respells
    the whole entry); the set is found over the face's packed names, a linked shared face's
    files and the player's own together; a variant whose `.mtl` is not the one the entry
    implies is `kit_variant_mtl_differs`; a `.mtl` texture path naming `pants_kitN` is pointed
    at the texture home. Still refused by name: a per-kit model under `ingame_face`, in a
    shared boots or gloves folder, or behind a link. Next: 4.14e5.
  - **4.14e5 done** (TC-MOD-44, TC-MOD-45), the last slice of 4.14: under `ingame_face` on
    PES 15-17 a `.model.common` link's Common model is copied in as one more part of the
    player's boots merge or one more model of his gloves, with the `.mtl` its search finds,
    and a `.mtl.common` link one of his parts uses is copied the same way; a copied Common
    `.mtl` names the Common textures in the team's Common output (decision entry). 4.14 has
    no refusal left for a pre-Fox player folder but a member's own `face.xml` (4.15). Next:
    4.15.
  - **4.15 recon**: the `mtl_material_duplicate` and `mtl_state_*` rows were already reported
    by the deep pass from `pes_model::check` (4.14d), so 4.15 is the member's own `face.xml`
    and `mtl_texture_not_found`; three items deferred with reasons in the step. Red keeps a
    member's xml and its file names as written (the pre-Fox tracer's Fumos folder: Red's output
    names `./face_high_*.model` beside `face_high_win32.model`), so a `./` reference packs its
    file under the referenced name and a Common reference is written as the Common output packs
    the model (decision entry; plan paragraph "How a `path` or `material` is resolved"). The
    step is sliced (a1) read/check, (a2) emit, (b) textures.
  - **4.15a1 done** (TC-XML-03, TC-XML-04 first half, TC-XML-06, TC-XML-09): a member's own
    `face.xml` is parsed, its references resolved and every content check of the table
    reported by the deep pass, which compares a listed model's materials against the xml's
    `.mtl` rather than the search's; the folder has a face; on Fox the xml is ignored with
    `xml_ignored_fox` and the gate lets it through; the pre-Fox gate still names it (a2 emits
    it). Next: 4.15a2.
  - **4.15a2 done** (TC-XML-01, TC-XML-02, TC-XML-04, TC-XML-05): on PES 15-17 a member's own
    `face.xml` compiles: written back in the generated shape with its attributes and unknown
    elements verbatim, a `./` file packed under the referenced name, a Common model written as
    the Common output packs it, the dummy and the PES 2015 rewrite as for a generated xml, its
    own `<dif>` else the folder's; only the files it names are packed; no hand split for such a
    folder; the pre-Fox tracer's Fumos xml compiles verbatim as a test. TC-XML-07 is 4.15b's.
    Next: 4.15b.
  - **4.15b done** (TC-XML-07; 4.15 closed): on PES 15-17 the deep pass looks every `.mtl`'s
    texture paths up by the face task's stem rule, `check` and `compile` alike: a miss on a
    material a paired model's mesh binds is `mtl_texture_not_found`, a Warning keeping the
    folder (the plan's Error refused the cup-played Fumos tracer; Red warned on the same file),
    a miss only unbound materials name `mtl_texture_unused_missing`; a Common-form path is
    looked for in `Common/` and the installed CPKs listed before the run's. Deferred: a pre-Fox
    texture link satisfied by an installed CPK; the `XXX` segment of a supplied Common-form
    `.mtl` path left as written. Next: 4.16.
  - **4.16a done** (TC-KIT-19, 21, 22, 23, 26, 28, 29, TC-BIN-04): PES 15-17 compile kits: the
    gate names only a kit texture no engine builds (a stem outside the seven, shared with the
    Fox walk), the textures go out as DDS under `common/character0/.../uniform/texture/`, the
    mask the kit's own or the bundled template byte for byte, the srm dropped with
    `kit_texture_not_used` (the mask on Fox), the main texture and the target's map re-laid by
    the marker and reported once per kit, the loose config the kit's only one (no
    `UniformParameter.bin`), the two color bins as on Fox and none of the Fox player bins.
    Red's mask template is bundled with its provenance (`resources/kits/README.md`). Next: the
    pre-Fox tracer's kit parity rows (lead), then 4.16b.
  - **4.16a parity** (lead): the pre-Fox tracer's twin gains `Kits/g1/`, and the parity test's
    four kit rows compare: the mask byte for byte, the texture by pixels, the config outside
    the four number-texture names Red keeps for textures the export does not ship (a question
    for the maintainer, in the 4.16 step), `UniColor.bin` as the bundled base with Red's `g1`
    entry set (Red's fallback base held an empty record for the team).
  - **4.16b done** (TC-TEX-08): `dds_compression` (`auto` follows `multicpk_mode`, `true`,
    `false`) WESYS-zlibs every `.dds` a PES 15-17 run emits, on the worker that made it; a
    wrapped source the conversion would keep is emitted as it is; Fox ignores the setting.
    Next: 4.16c (collars for pre-Fox), 4.16d (installed loose kit configs patched).
  - **4.16c done** (TC-CMN-08): a team's `Collars/collar_<ID>.model` compiles for PES 15-17 as
    its FMDL does on Fox, written unchanged in place of the stock collar, the claim and the
    conflict rules the same; a `.fmdl` collar is named there until 4.17 converts it. Next:
    4.16d.
  - **4.16d done; 4.16 closed** (TC-BIN-18): on PES 15-17 a `Midcup` export's absent kit slots
    have their installed loose configs gathered by the working-bin walk, FPC-patched and
    collar-worn as the `UniformParameter.bin` entries are on Fox, and the changed ones go out
    through the bins path (DECISIONS 2026-10-08). Next: 4.17 (cross-format conversion and
    source selection).
  - **4.17-lead** (recon `.tmp/4_17/recon_4_17.md`, Opus, read-only): the step's text widened
    (TC-MOD-26's PES 17 half, same-engine retargeting, the loss-code mapping, shared folders
    and links as slice f), the source-selection rulings logged (DECISIONS 2026-10-08), the
    template environment cubemap bundled with its provenance. Next: 4.17a (a Fox face
    converted for PES 17 through the face task).
  - **4.17a done** (TC-MOD-27): on PES 15-17 a player's `.fmdl` with no `.model` twin is
    converted through `model_convert` and packed by the face as a `.model` and `<stem>.mtl`,
    its `.skl` the bind pose; the Fox tracer compiles whole for PES 17 and PES 15. The game's
    dummy textures cross engines by rule (DECISIONS 2026-10-08): the FMDL import gives
    `dummy_nrm`/`dummy_srm` no role, `dummy_kit*` is pointed at the team's Common directory.
    Next: 4.17b (pre-Fox to Fox in the Models task, the selection rule on Fox).
  - **4.17b done** (TC-MOD-26, 29, 30, 34): on PES 18-21 a player's `.model` with no `.fmdl`
    twin is converted in the Models task with the `.mtl` its search finds, the skeleton the
    conversion writes taking a member's `.skl` path; every converted model is checked in its
    target form, a failure leaving its package out as every task failure does (DECISIONS
    2026-10-08); a beaten model is read by nothing, and a selected `.model`'s `.mtl` gets the
    texture check on Fox. Next: 4.17c (every loss code mapped, Warnings of the converted form,
    the same-engine pre-check, `model_gltf_unsupported`).
  - **4.17c1 done**: every `model_convert` loss code is reported at a catalog severity
    (Warning for a change in how the model looks or moves, Info for a drop the game cannot
    show), the converted form's own Warnings and Infos are not (DECISIONS 2026-10-08). The
    same-engine pre-check was wired as the plan said and measured: `needs_conversion`'s
    reference poses disagree with the pre-Fox parity tracer on Red's own target and fire on
    Fox only for boots shipping the game's `boots.skl`, so it is deferred to slice (g), a
    two-crate step reconciling the poses first (DECISIONS 2026-10-08). Next: 4.17c2 (a
    selected glTF refused at planning, TC-MOD-28).
  - **4.17c2 done** (TC-MOD-28): a `.glb`/`.gltf` with no model of the target's own format
    of its stem beside it is its stem's selection, beating the other engine's model; planning
    drops its player folder with `model_gltf_unsupported` instead of the gate refusing the
    export, never compiling the beaten model in its place; a glTF the target's format beats
    is ignored. A shared folder's glTF stays with the gate until slice f. Next: 4.17d (the
    template environment cubemap for a converted metal material, TC-MOD-36).
  - **4.17d done** (TC-MOD-36): a Fox metal material converted for PES 15-17 gets an
    `EnvironmentMap` sampler naming `env.dds` in the player's texture home, where the
    textures task emits the template cubemap unless the folder holds an `env` texture of
    its own; the deep pass records the metal FMDLs with the converter's own family rule and
    planning flags the folder. Decision logged. Mutants: 27, 21 caught, 6 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17e
    (collars across engines, TC-CMN-09; plan edits and the rulings first).
  - **4.17e done** (TC-CMN-09): an FMDL collar compiled for PES 15-17 is converted with the
    stock collars' material names and no `.mtl`; a glTF collar is dropped at planning with
    `model_gltf_unsupported`, the file alone; a `.model` collar for PES 18-21 stays with the
    gate. Two decisions logged. Mutants: 24, 19 caught, 5 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f (shared folders,
    `Common/` models and `.common` links in the other format, the shared folder's glTF, the
    `ingame_face` FMDL parts, the remaining 4.17 open items; sliced after an exploration).
  - **4.17f1 done**: a shared folder's `.model` on PES 18-21 and a shared face's FMDL on
    PES 15-17 convert as a player's own, the gates no longer refusing them; a shared folder
    whose selected model is a glTF is removed at planning and every player linking it is
    dropped with `model_gltf_unsupported` naming the shared file. Decision logged. Mutants:
    27, 27 caught, 0 unviable, 0 missed. Next: 4.17f2 (a shared boots or gloves folder's FMDL and an `ingame_face`
    player's FMDL parts converted by the pre-Fox boots and gloves writer; the two plan gaps
    from f1).
  - **4.17f2 done**: on PES 15-17 a shared boots or gloves folder's FMDL and an `ingame_face`
    player's FMDL parts convert in the boots and gloves writer as the face converts one, the
    gates no longer refusing them; a conversion finding names a shared source by its export
    path; role-less Fox files in a shared folder ignored as a player's. Decision logged.
    Mutants: 60, 57 caught, 3 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3 (`Common/` models and `.common` links in the other
    format, per-kit `.model` variants on Fox, the pre-Fox Common `.mtl` path on Fox,
    `kit_variant_model_fox` on a shared folder linked only by a dropped folder, the `env`
    link ruling, the remaining `model` contexts, the env-sampler asymmetry).
  - **4.17f3a done**: the four leftovers of 4.17f: planning's glTF drop removes the shared
    folders it orphans; a player's `env.dds.common` link is the environment map his
    converted metal materials name, no template emitted and no empty textures task planned
    (one planned lost his face CPK silently: the writer takes an empty batch for a failed
    task's, now an open converge item); every `model` context of a folder's task named
    through `source_name`; the boots and gloves writer adds the environment sampler as the
    face does; `kit_variant_model_fox` silent on pre-Fox, where the set is packed whole
    (lead fix from the sidekick's probe). Mutants: 19, 15 caught, 4 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3b (an FMDL
    in `Common/` converted once in the Common models task on PES 15-17, the `x.fmdl.common`
    link listing its `.mtl`, the marker case).
  - **4.17f3b done**: on PES 15-17 an FMDL directly in `Common/` converts once, in the
    Common models task, into the team's Common output with its material set as
    `<stem>.mtl`, its `.skl` the bind pose and the environment map emitted there for a metal
    one; an `x.fmdl.common` link lists it at the Common path with that `.mtl`, and under
    `ingame_face` converts into the part; a link to a Common FMDL a Common `.model` of its
    stem beats loads the `.model`. Decision logged. Mutants: 52, 43 caught, 9 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3c
    (PES 18-21: a Common `.model` + `.mtl` converted in each linking player's Models task;
    per-kit `.model` sets; the pre-Fox Common texture path pointed at the Fox Common
    directory).
- **2026-10-08** — `.tmp/` sorted (maintainer's request): one folder per subphase (`4_14a`
  and `4_14b` under `4_14/`), `lead/` for the lead's live notes, `review_queue_S/` for the
  S1-S6 review rounds, `reference/` for cup material; files with no step in their name went
  by the subphase the log mentions most on their mtime day (about 550 of 2063 entries, so
  some sit a step off). Every `.tmp/<name>` reference in the docs, `AGENTS.md`, the fixture
  READMEs and `scripts/provenance/` repointed (85 + 7). Deleted as regenerable: every cargo
  `target/` under `.tmp/` but the apptest tool's, and 4.7's laid-out timing exports and
  outputs (`timing_build.py` rebuilds them): 5.9 GB of 7.0. What else can go is checked at
  the end of the implementation work, not now. Scripts and the move list in `.tmp/sort_tmp/`.
- **2026-10-08** — Test 4c (in-game, PES 17; Issues "pre-Fox player tables", the Test 4b
  paragraph): no. 1's PlayerGloves bit set on his DB appearance row changed nothing, bare
  hands in Edit mode (lead) and in a match (maintainer). A GloveList.bin row never applied
  on PES 17; pre-Fox gloves stay in the savefile. The maintainer settled that Edit mode shows
  a player as a match does (only outfielders' 2nd+ kits need a match), so in-game look tests
  stop at Edit mode.
- **2026-10-08** — `docs/QUESTIONS.md` created (maintainer's request): the one list of the
  questions only he can answer, fourteen moved out of the worklog's steps and issues (each
  of those now points there); `AGENTS.md` "Working documents" gained the row.
  - **4.17f3c done**: on PES 18-21 a `.model` directly in `Common/` with its `.mtl`, linked
    by `x.model.common`, converts in each linking player's Models task as his own `.model`
    does, an FMDL of its stem beating it, its Common `.mtl` paired and looked up in the deep
    pass on the player's folder; a per-kit model set's variants are the folder's `.fmdl`
    and `.model` files, a number in both formats one variant with its FMDL selected; a
    converted path naming the pre-Fox Common texture directory is pointed at the Fox Common
    directory when `Common/` holds the stem. Two contradictions accepted (the finding names
    the Common model's export path; the fixture). Mutants: 76, 62 caught, 13 unviable
    (`Default::default()` on types without one, `||` in conditions the types refuse), 1
    missed (`model_variant_sets`'s dedup rule for two files of one number and format),
    caught by the lead's added assertion (rerun on the function: 5 caught, 1 unviable).
    Next: 4.17g (the same-engine pre-check of a selected native model).
  - **4.17g1 done** (measurement, no code): which pose each real model is bound to, on the
    games' own files (stock PES 2017 faces, boots and gloves out of `dt33_win`/`dt36_win`,
    stock PES 2021 boots and gloves FPKs out of `dt33_g4`) and the tracers
    (`.tmp/4_17/sk_4_17g1_report.md`). Faces carry a per-face `skf_*` pose (Konami's up to
    0.34 off `face.skl`); boots sit on one boots pose 7° off the body on every version, both
    feet in one mesh; the pre-Fox tracer gloves sit on PES 15's arm pose with every bone
    sharing one delta; a stock glove is 0.0025 off the hand table, the smallest version
    difference 0.0042. Ruling (DECISIONS 2026-10-09): the pre-check tests whether a vertex
    blends bones with differing deltas, `skf_*` take no part, tolerance 3e-3; no boots table.
    Fixtures copied by the lead into `model_convert/tests/fixtures/` (stock PES 17 boots and
    glove, stock PES 21 boots with its six-bone `.skl`, the tracer's three models). The
    2026-10-08 ruling's three comparisons are withdrawn. Next: 4.17g2 (`model_convert`).
  - **4.17g2 done**: `model_convert::needs_conversion` is the plan's blended-delta test (a
    used bone the target lacks, or a vertex blending bones whose deltas to the target's table
    differ beyond 3e-3; `skf_*` and custom bones take no part), shared by both engines through
    one pass over the weight rows; `retarget` keeps a face's `skf_*` matrices; the tolerance
    3e-3. On their own version the stock PES 17 boots and glove, the stock PES 21 boots with
    its `.skl`, the tracer's boots, gloves and face and the one-bone card are all `false`
    (every one was `true` before). One contradiction accepted: a pre-Fox weight below half a
    Fox step (0.5/255) is no weight, since the tracer's boots carry 2^-24 of noise on the
    other leg's bone (plan item 3, DECISIONS 2026-10-09). Side effect pinned by a lead test:
    a converted face now writes an SKL carrying its own `skf_*` pose (`needs_skl`), where the
    re-bind used to hide it. Mutants: 23 (19 caught, 3 unviable, 1 missed: `slot + 1` to `slot * 1` in the pair loop of `blends_differing_deltas`, equivalent, since a slot compared with itself never differs). Next: 4.17g3a (`team_compiler`, Fox).
  - **4.17g3a done**: the Fox Models task runs `needs_conversion` on every selected FMDL (a
    player's own, a shared folder's, a `.common` link's) with the `.skl` of its stem as the
    bind pose, through one `convert_part` for both formats (`conversion::fmdl_for_fox`;
    `fox_written` shared with `model_for_fox`); a flagged FMDL is moved onto the version's
    skeleton (`skeleton_retargeted`), its `.skl` dropped for the conversion's; skeletons are
    read before the models; the gloves task's files include a hand-split part's `.skl`
    (`ModelFolder::hand_split_skeleton`), so its hands are moved as the face's body is. The
    skeleton a conversion writes for a slotless role is dropped with no finding, `skl_no_slot`
    staying a member's-file warning (every converted face writes one since g2: TC-MOD-48's red
    run showed the warning at HEAD). Collars are not pre-checked. Lead measurements
    (`.tmp/4_17/lead_g3a_probe/`): the Fox tracer's four FMDLs with their own, PES 18's or
    PES 19's skeleton, the hand-split body and every `fmdl` fixture are `false` on PES 18-21,
    so only a moved blended bone flags a Fox model. TC-MOD-46, 47, 48 added and proven. One
    test's input replaced (the linked Common model's export-path naming, now through a renamed
    bone's `skeleton_retargeted`). Mutants: 29 (21 caught, 8 unviable: type-driven defaults and an `&&` inside a let chain, none missed). Next: 4.17g3b (`team_compiler`,
    pre-Fox).
  - **4.17g3b done**: `conversion::model_for_pre_fox` (the `.model` taken, the bytes to pack
    returned: the source as it is, or the model moved onto the version's skeleton), its
    writing half `pre_fox_written` shared with `fmdl_for_pre_fox`; called by the face
    (`FaceSource::Member`), the shared boots and gloves writer (`prefox_shared::package`, both
    arms) and the Common models task; the face task's files include the Common `.mtl` each
    `.mtl.common` link names (`plan::linked_common_materials`), so a model with a Common
    `.mtl` is checked and converted too, and the hand split of such a model is no longer
    refused (DECISIONS 2026-10-09); a `Common/` `.model` with no `.mtl` in `Common/` is packed
    as it is (sidekick's finding: the `expect` fired on a linking player's own `.mtl` and on an
    unlinked model). The cap folds two bones on PES 15 and re-binds none, so the tests pin the
    fold lines, not `skeleton_retargeted` (TC-MOD-49 to 52 reworded). Two rework rounds: R1 the `model_for_pre_fox` shape (the bytes to pack returned) and the hand split over a Common `.mtl` allowed, both from the sidekick's own report; R2 one test for the mutation run's survivor.
    Mutants: 26 (21 caught, 5 unviable: type-driven defaults; the first run's one survivor, the dedup in `linked_common_materials`, caught by R2's two-links test). Next: 4.19d.
- **2026-10-09** — Reviews run by the lead from now on: the maintainer wired the `duck` skill
  (Astra, SWE-2 rounds) for Claude Code, so cross-family reviews are no longer queued except on
  an Astra quota stop (five-hour quota, five or six reviews; retried hourly). The S7-S16
  backlog and the four queued 4.14/4.17 reviews run as the first half of converge, after the
  remaining slices (maintainer: finish the crate first). "Handover" rule and `AGENTS.md`
  "Second opinion" reworded. 4.17g started as g1, a measurement brief (`.tmp/4_17/brief_4_17g1.md`):
  which game table each real model is bound to (the tracers, stock PES 17 and PES 21 models from
  the installs, the converter's fixtures, the body-table version deltas), so g2 changes the
  converter from numbers, not from a reading of its code. The maintainer also noted that the
  in-game harness on PES 17 and 21 can answer `docs/QUESTIONS.md` entries without them.
- **2026-10-09** — 4.17g1 (measurement): the sidekick's probe (`.tmp/4_17/sk_4_17g1_probe/`)
  measured every real model's bind pose against the tables; the 2026-10-08 ruling (face table,
  `boots.skl`, hand pose) is withdrawn and the pre-check becomes a blended-delta test
  (`conversion.md` "When re-binding changes anything", DECISIONS 2026-10-09); fixtures added.
  Test 3 on PES 17 run through the harness as far as Edit mode allows: team /a/'s outfielders
  are all billboard or custom-body models, so kit 1's socks and collar 200 cannot be seen there
  (only a match would); the keeper's collar 201 (no model anywhere) drew a plain neckline, no
  crash. The PES 21 half and Test 3b are left for the maintainer; the apptest scripts'
  absolute `.tmp/apptest` paths were repointed to `.tmp/4_0/apptest` (missed by the sort).
- **2026-10-09** — 4.17g2 landed (sidekick, first time; one contradiction accepted, the
  pre-Fox weight floor): `needs_conversion` as the blended-delta test, `skf_*` out of the check
  and the re-bind, tolerance 3e-3, seven real fixtures `false` on their own version. Lead test
  added (no red run; it pins the `needs_skl` side effect: a converted face's SKL carries its
  own `skf_*` pose). Gates 255 proven. Mutants: 23 (19 caught, 3 unviable, 1 missed: `slot + 1` to `slot * 1` in the pair loop of `blends_differing_deltas`, equivalent, since a slot compared with itself never differs). The FMDL importer's
  `bone_matrices` reading filed for converge.
- **2026-10-09** — 4.17g3a landed (sidekick, first time; one test input replaced, reported):
  the Fox pre-check call per selected FMDL with its `.skl`, the gloves task reading a
  hand-split part's `.skl`, the slotless conversion skeleton dropped silently (the g2 side
  effect would have warned `skl_no_slot` on every converted face: proven by TC-MOD-48's red
  run), collars not pre-checked; two decision entries, plan edits first; TC-MOD-46 to 48.
  Gates 258 proven. Mutants: 29 (21 caught, 8 unviable: type-driven defaults and an `&&` inside a let chain, none missed). The `bone_matrices` converge item extended
  (exporter writes the block, `needs_skl` then skips `skf_*`).
- **2026-10-09** — 4.17g3b landed (sidekick; two rework rounds: the `model_for_pre_fox` shape
  and the hand split over a Common `.mtl` allowed, both from its own report; then one test for
  the mutation run's survivor, two `.mtl.common` links to two Common files): the pre-Fox
  pre-check call for every packed `.model`, the member's `.mtl` kept, the Common `.mtl` a link
  names read by the face task; three contradictions accepted (the cap folds and re-binds
  nothing, the IR drops `.model` mesh tags, the hand-split refusal lost its reason), one plan
  gap accepted (a `Common/` `.model` with no `.mtl`); three decision entries. Gates 262 proven.
  Mutants: 26 (21 caught, 5 unviable: type-driven defaults; the first run's one survivor, the dedup in `linked_common_materials`, caught by R2's two-links test). 4.17's slices are done; converge items filed (`.model` collars,
  mesh tags, `bone_matrices`).
- **2026-10-09** — 4.20 ruled and 4.20a landed. Lead: a reconnaissance (Opus agent,
  `.tmp/4_20/recon_4_20.md`) of what validation, planning and processing do with each class
  the gate named, eight decision entries, the plan edits, TC-OUT-06 withdrawn and eleven IDs
  added; the pre-Fox referee template tree bundled for 4.19d. Sidekick: the removal and
  `file_not_used`; one tests-only rework round (the mutation run's two survivors) and one lead
  fix (the pre-Fox list of Common files a face reads made direct-only, which R1's test showed
  nested); eight contradictions, one applied (the deep
  pass's search resolving a shared folder's `.mtl` link, a panic), one deferred to b1 with a
  decision (a Common-set part's `./skin.dds`), six accepted (material count, TC-OUT-02's inputs,
  a line's form, the glTF rule's reading, which becomes b1's, face files in boots folders, the
  help). Gates 268 proven. Mutants: 78 (62 caught, 16 unviable: type-driven defaults and an && inside a let chain; the first run's two survivors, the Fox Common .mtl filter's && and admitted, caught by R1's tests).
- **2026-10-09** — 4.19d lead-first: the pre-Fox referee template tree bundled
  (`resources/templates/referees_prefox/`, 51 files from Red `e12aa01`, byte-identical to
  Blue's), its provenance in the templates README, the tree marked `-text` in `.gitattributes`
  (three CRLF `.mtl` files). Nothing reads it yet: 4.19d chooses the tree by engine.
- **2026-10-09** — 4.19d lead-first (2): `resources/templates/collar_empty.model` bundled (FPC's
  empty collar, from `4cc_04_fpc.cpk`, provenance in the templates README); the marker paragraph
  of `blue_port.md` names the `referee_prop` pair and the empty collar; decision entry; 4.27's
  pre-Fox half folded into 4.19d; brief drafted.
- **2026-10-09** — 4.20b1 landed (Opus sidekick, one round, no lead fix): unused kit stems,
  `shared_folder_no_model`, a Common-set part's textures, the Common glTF order. One
  contradiction applied by the sidekick and accepted with a decision entry (a player whose link
  names a beaten Common model is dropped: the brief's shape panicked in every link reader), four
  accepted (a selected glTF counts as a model, roster order is slot order, the readers of the
  removal, TC-MOD-54's THEN), three observations sent to b2's brief (the help's gate passage,
  `shared_folder_no_model` in the help, a refs kit reported twice). Gates 270 proven. Mutants:
  51 (38 caught, 12 unviable: type-driven defaults; one survivor, the `||` of `linked_common_gltf`'s link filter, caught by a lead input added to the Common glTF planning test, a `.mtl.common` link whose stem a selected glTF holds, which must not drop its player; the kill checked by a hand perturbation).
- **2026-10-09** — 4.20b2 landed (Opus sidekick, one round, no lead fix); 4.20 closed: per-kit
  model sets left out on PES 15-17 where no `face.xml` names them (the finding renamed
  `kit_variant_model_left_out`, the set's selected representation by the target's format), a
  shared folder's `face.xml` ignored with `xml_ignored_shared`, the help's gate passage gone, a
  refs export's kits left to validation. Seven contradictions, none applied against the brief:
  one widened the rule's plan text (any shared folder's xml), one recorded in the plan (a lone
  variant under the marker is a part), one brief error (ten callers, not nine), four accepted
  as written. Gates 272 proven. Mutants: 61 (47 caught, 14 unviable: type-driven defaults and an && inside a let chain, 0 missed).
- **2026-10-09** — 4.19d landed (Opus sidekick, one round, no lead fix); 4.27's pre-Fox half
  with it: a PES 15-17 refs CPK carries the pre-Fox template tree (51 files, overridable from
  `templates/referees_prefox/`), and `ref_marker.dds` goes out as `referee_collar_077.model`
  (the tree's prop model), its `.mtl` holding `judge_incom` alone pointed at the marker
  texture, and the bundled empty `collar_077.model`. Four contradictions accepted (TC-REF-09's
  wording twice, the pre-Fox prop unskinned rather than painted to `static`, the pre-Fox
  configs' collar 26); the TC-REF-10 probe opened 4.19e (a pre-Fox referee's shared boots
  written twice). Gates 274 proven. Mutants: 55 (30 caught, 25 unviable: type-driven defaults and leaked-static replacements of the tree accessors, 0 missed).
- **2026-10-09** — 4.27b landed (Opus sidekick, one rework round, the multi-CPK refs-only test the sidekick itself flagged as uncovered; no lead fix); 4.27 closes: on
  PES 18-21 every referee kit config written to the refs CPK is also the entry of its file
  name in the run's `UniformParameter.bin`, with the bytes written (collar 77 under the
  marker, a `templates/referees_fox/` replacement's bytes), so a run whose only committing
  export is the refs export writes the team side for the bin (multi-CPK: the bins CPK and
  the placeholder parts); PES 15-17 unchanged. One contradiction accepted: the bundled
  bases hold no `referee_1..5` entries (a byte grep had read kit texture names inside the
  configs as entry names); the `LB` and `SDA` entries are new, inserted by name. Gates 275
  proven. Mutants: 10 (8 caught, 2 unviable: type-driven defaults of finish_team's return, 0 missed).
- **2026-10-09** — 4.19f landed (Opus sidekick, one round, no rework; lead fix: a test name
  shortened): on PES 15-17 `ref_marker.dds` goes out converted as the template tree's
  `parts/referee/incom_bsm.dds`, the texture of the prop the game draws under the referee,
  in place of the tree's file (skipped in `finish_referees` as an override's is); no collar
  pair, no `collar_077.model`, the configs the template's; `collar_empty.model` and its
  resource removed, one marker path per engine (`paths::referee_marker`). Six contradictions
  accepted (an `Engine` match, not `==`; `referee_tree_file` removed with its last caller;
  a help sentence and a README sentence the change made false; the tree's own
  `incom_bsm.dds` is WESYS-compressed where the compiler writes a plain DDS, as Red did,
  noted in the plan and the open question; an over-long test name). Gates 275 proven.
  Mutants: 15 (13 caught, 2 unviable: type-driven defaults of the Referees constructors, 0 missed).
- **2026-10-09** — 4.19e landed (Opus sidekick, one round, no rework; lead fix: the help's
  referee paragraph): on PES 15-17 a referee's link to a shared `Boots/` or `Gloves/` folder
  is written as his slot's `k99NN`/`g99NN` folder alone, no longer copied into his face as a
  `parts` entry (the pre-Fox face reads a combined boots or gloves source's texture stems
  alone; planning's file list likewise); a referee folder with no face model gets no face
  folder on either engine, the game's referee head staying (decision entry: a referee has no
  FPC body to bring a head). Two of the brief's rulings corrected by the sidekick on
  evidence, both accepted. TC-REF-13 proven, TC-REF-10 extended; the Phase 4 steps are all
  done but converge. Gates 276 proven. Mutants: 9 (7 caught, 2 unviable: type-driven defaults of TaskKind::files and plan_run, 0 missed).
- **2026-10-09** — Converge (4.y-conv) opened: the 4.14 and 4.17 parent bullets closed with
  their verify criteria re-run (TC-MOD-20, 26, 27, 28 tests green). The lead's audit runs
  first (`.tmp/lead/converge_4_plan.md`), the `duck` reviews after it.
- **2026-10-09** — 4.y-conv, the lead's audit done: 64 verify criteria re-run (55 proven, 3
  without a proving test, 1 stale), TC-OUT-18 proven by a lead test, two catalog rows
  withdrawn, 6 Clef flags ruled (1 accepted), the export census over 71 roots (55 old-layout
  exports, 16 referee exports) diagnosed into eight classes. Three gap steps added above
  4.y-conv (small fixes; `export_layout_old` and the dual-engine face diff, two decision
  entries; cube-map textures), four issues and two maintainer questions logged.
- **2026-10-09** — 4.y-fix1 done (sidekick, landed first time): the eight converge fixes;
  three issues settled by tests, one of them red (a Fox `ingame_face` player's own texture
  counts for his boots, `merged_texture_conflict`). Whole-crate mutation runs of
  `aesthetics_export` (548 mutants, 2 survivors) and `pipeline` (72, 6 survivors, all one
  platform read) done and triaged into 4.y-fix2 (the `pipeline` six turned out to be the
  platform arms `.cargo/mutants.toml` already lists as accepted, each half running the other
  platform's body: 4.y-fix2's report); `team_compiler`'s run next, over this tree.
  `team_compiler`'s run done 13:34 (2,222 mutants over `23f68ae`, split: 1,111 a half, 2 h 25
  local and 2 h 39 remote, the remote peak 9.00 GiB of 9G with no build killed; 1,866 caught,
  347 unviable, 3 timeouts, 6 survivors; `.tmp/4_y/mutants_team_compiler/`): the timeouts
  are an atlas eight times too long (`kit_layout.rs:331` `/`→`*`), an empty range that
  never advances (`writer.rs:213` `+`→`*`) and an inverted poll (`gui_run.rs:52`), all
  explained; the survivors: `team_assets.rs` `centre_square`/`letterboxed` `>`→`>=`
  equivalent (4.11a's ruling, now in `.cargo/mutants.toml`), `deploy.rs` three error-kind
  guards (`sweep_run` 180, `probe_download` 406 and 409) missing tests → 4.y-fix2 (e), and
  `lib.rs:75` `settings_view`, a placeholder label no test reads until Phase 8 builds the
  settings view (left as a missing test for that step).
- **2026-10-09** — 4.31's verify criterion run at converge: Red rebuilt from the fixture's
  `old/`, every entry byte-identical to `red/` (the nested CPK's timestamp aside). 4.y-fix3's
  recon done and ruled: a cube-map DDS goes out as Red ships it on each engine (decision
  entry; a maintainer question on the Fox form). The `team_compiler` whole-crate mutation run
  started over `23f68ae`.
- **2026-10-09** — The `duck` reviews started: S7 and S8 (briefs `.tmp/duck_brief.md`,
  `.tmp/duck_brief_S8.md`, pointers only, the current code as the surface) launched in
  parallel at 11:11 and 11:15, both stopped at 11:16 by Astra's five-hour quota
  mid-review with no report; retried from 14:50 one at a time, the queue as in
  "Handover". 4.c-threshold done meanwhile: 0.7 kept (decision entry).
- **2026-10-09** — 4.y-fix2 done (sidekick, landed first time, two contradictions accepted:
  the `pipeline` survivors were the documented platform arms; two of the three `deploy.rs`
  guards are killed on Windows already). An old-layout export is `export_layout_old` alone;
  a `face.xml` `<dif>` beside a `face_diff.bin` is each engine's own. TC-ROOT-14 and
  TC-XML-11 proven. The census re-run follows the commit.
- **2026-10-09** — 4.y-fix3 done (sidekick, one rework round from its own contradiction: the
  FTEX writer and the compiler's cube test now read cube-ness by one predicate). A cube-map
  DDS is never decoded: as it is on PES 15-17, `dds_to_ftex`'s type-0xD cube map on PES
  18-21. TC-TEX-13 proven; acceptance 280 of 280. The verify re-run follows the commit.
- **2026-10-09** — The `duck` review S7 ran at 14:52 (`198711a`, six minutes): four concerns,
  three accepted (a panic when a linked face's `.model` boots convert without their
  `.mtl`; a boots or gloves link never combining with a linked face's parts or the split
  hands; the plan's tolerance rule for IR-derived skeletons, narrowed by a decision entry
  rather than built), one rejected (one Common model linked twice from one player).
  Rulings `.tmp/4_y/duck_rulings.md` S7.A1. S8's run at 15:00 stopped on the five-hour
  quota again ("try again at 4:44 PM"): the reset bought one review; retried hourly.
- **2026-10-09** — 4.y-fix4 done (sidekick, landed first time): S7's two code concerns.
  TC-MOD-58 to 60 proven; acceptance 283 of 283. One open issue filed (the deep pass's
  link answer without the hand-weighted set).
- **2026-10-09** — The `duck` review S8 ran at 16:45 (`f65c07c`, five minutes): four concerns,
  three accepted (a Common model converted with the player's local `.mtl` pointing its
  textures at Common; a signed-format DDS in `Common/` failing the whole Common textures
  task; an FTEX portrait keeping its single level), one rejected (S7.A1-4 again). Rulings
  `.tmp/4_y/duck_rulings.md` S8.A1. S9's run at 16:52 stopped on the quota ("try again at
  9:45 PM"): one review per reset so far.
- **2026-10-09** — 4.y-fix5 done (sidekick, landed first time): S8's three concerns.
  TC-MOD-61, TC-TEX-14 and TC-TEX-15 proven; acceptance 286 of 286.
- **2026-10-09** — The Astra reviews moved to the Devin CLI (`duck.py --reviewer astra-devin`,
  read-only, the user's three Devin logins rotated by `devin_account.py`; no Codex call until
  the user says otherwise). S9's three runs that evening all died on the accounts' daily
  usage (one review of this size exceeds a Pro plan's day; no export is written when a run
  dies mid-turn), so S9 stays queued. Step 4.y-fix6 opened for the VGL stream's PES 19
  crash on a DX10-header portrait (maintainer's report).
- **2026-10-09** — 4.y-fix6 done (sidekick, one rework round): a DX10-header DDS portrait
  re-headered, `portrait_header_rewritten`. TC-PRT-04 proven; acceptance 287 of 287. The
  maintainer traced the header to paint.net's "BC3 (sRGB, DX 10+)" save option. The reviews
  wait for the maintainer's instructions (every Devin daily quota spent on S9's attempts).
