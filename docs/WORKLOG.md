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
4.y moved to Phase 8; 4.14's slices are all done (a to d, e1 to e5; its GPT reviews (a) TC-MOD-43 and (b) e3 stay queued); 4.15 done (a1, a2, b: the member's own `face.xml` read, checked and emitted; `mtl_texture_not_found` a Warning on pre-Fox, Fumos's evidence); 4.16 done (a: PES 15-17 kits with the mask template and loose configs; b: `dds_compression`; c: `.model` collars; d: the installed loose kit configs of absent slots patched and re-emitted), with its own checks until 4.31's pre-Fox parity; 4.17 in progress (lead done: rulings and the environment cubemap; a done: a player's FMDL converted for PES 15-17 through the face task, its GPT review (c) queued; b done: a player's `.model` converted for PES 18-21 in the Models task, every converted model checked in its target form, a beaten model read by nothing; c1 done: every conversion loss reported at its catalog severity, the same-engine pre-check measured and deferred to slice g; c2 done: a selected glTF in a player folder refused at planning with `model_gltf_unsupported`, TC-MOD-28; d done: the template environment cubemap emitted for a converted metal material on PES 15-17, its GPT review (d) queued, TC-MOD-36; e done: an FMDL collar converted for PES 15-17 with the stock names, a glTF collar dropped at planning, TC-CMN-09; f1 done: a shared folder's other-format model converted as a player's, a shared folder's glTF dropping its linking players; f2 done: a shared boots or gloves folder's FMDL and an `ingame_face` player's FMDL parts converted by the pre-Fox boots and gloves writer, a conversion finding naming a shared source by its export path; f3a done: the glTF drop's orphans removed, an `env` link the environment map, every `model` context named alike, one sampler shape, `kit_variant_model_fox` Fox-only; f3b done: a Common FMDL converted once in the Common models task on PES 15-17, its link listing the conversion's `.mtl`; f3c done: a Common `.model` converted in each linking player's Models task on PES 18-21, per-kit sets of either format, a pre-Fox Common texture path pointed at the Fox Common directory; g1 done: the pose measurement on the games' own files, the pre-check's rule settled as a blended-delta test; g2 done: `needs_conversion` is that test, `skf_*` pass through, tolerance 3e-3; g3a done: the Fox Models task pre-checks every selected FMDL with its `.skl`, the gloves task reads a hand-split part's `.skl`, a slotless conversion skeleton is dropped silently; g3b done: every pre-Fox `.model` pre-checked with the member's `.mtl` packed beside a moved one, the Common `.mtl` a link names read; 4.17's slices are done); 4.20 in progress (lead rulings and a done 2026-10-09: the gate gone, a file no role reads `file_not_used`; b1 done: unused kit stems, `shared_folder_no_model`, Common-set textures, the Common glTF order; b2 done: per-kit sets left out on pre-Fox where no `face.xml` names them, a shared folder's `face.xml` ignored; 4.20 done); 4.19d done 2026-10-09 (with 4.27's pre-Fox half; the TC-REF-04 in-game check: checked 2026-10-09 by the lead on PES 17 through the harness (`.tmp/4_19/ingame/`, results in `.tmp/4_0/apptest/results.txt`): the referee present with the pair installed, nothing drawn; a `.mtl` beside a nocloth `.model` is read and stops the model drawing (runs E and F against 2026-10-08's run I), so the pre-Fox marker goes by Red's route instead: step 4.19f, DECISIONS 2026-10-09; the scene where the pre-Fox marker shows is a maintainer question); 4.27b done 2026-10-09 (the Fox referee configs also their `UniformParameter.bin` entries; a refs-only Fox run writes the team side); 4.19f done 2026-10-09 (the pre-Fox marker by Red's route: the template prop's texture replaced, no collar pair, `collar_empty.model` gone); 4.19e done 2026-10-09 (a referee's shared link his slot's folder alone on PES 15-17; no face folder for a referee without a face model); every Phase 4 step is done (the 4.14 and 4.17 parent bullets closed 2026-10-09, their verify criteria re-run); 4.y-conv in progress since 2026-10-09: the lead's audit done (verify re-run, acceptance, catalog producers, sweep, `pub` census, Clef, the export census), 4.y-fix1 done 2026-10-09 (eight small fixes, three issues closed by tests); the whole-crate mutation runs of `aesthetics_export` and `pipeline` done and triaged, `team_compiler`'s next; 4.y-fix2 done 2026-10-09 (`export_layout_old`, the dual-engine face diff, the survivors' tests), 4.y-fix3 done 2026-10-09 (a cube-map DDS goes out as Red ships it; TC-TEX-13, acceptance 280 of 280), 4.y-fix4 done 2026-10-09 (S7's rework: a linked face's `.model` boots convert with their `.mtl`, a boots or gloves link combines with the player's effective parts; TC-MOD-58 to 60), 4.y-fix5 done 2026-10-09 (S8's rework: a Common model's local `.mtl` resolves its textures in the folder, every DDS kind the decoder refuses is `texture_codec_unsupported`, an FTEX portrait gets the full chain; TC-MOD-61, TC-TEX-14, TC-TEX-15), 4.y-fix6 done 2026-10-09 (a DX10-header DDS portrait goes out re-headered: the VGL stream's PES 19 crash; TC-PRT-04, acceptance 287 of 287), 4.y-fix7 (a) done 2026-10-09 (the deep pass checks what
`compile` reads: S9's rework; acceptance 295 of 295; (b) done 2026-10-10, the pairing code
moved into `deep/pairings.rs`), 4.y-fix8 done 2026-10-09 (S10's and S11's rework: the absent slots' collar and clamp
reported, a `Collars/` subfolder ignored; TC-BIN-23, TC-BIN-24, TC-CMN-16, TC-REF-14),
4.y-fix9 done 2026-10-09 (a shared face's `face.xml` rules its linking faces; TC-XML-14,
TC-XML-15),
4.y-fix11 done 2026-10-10 (S7's, S8's, S12's and S13's SWE-2 rework: fourteen items; a kit
variant set spans every file a part is built from),
4.y-fix15 opened 2026-10-10 (S14's SWE-2 rework), 4.y-7z done 2026-10-10 (a small `.7z`
export decompressed once: kept from its check to its tasks under an eighth of the cap, the
floor and the run's largest other archive left free),
4.y-fix10 done 2026-10-10 (a pre-Fox per-kit model set completed against the kit
numbers; TC-CMN-17), 4.y-harness and 4.y-ingame done 2026-10-10 (the converted collar and `dummy_kit`
checked in game, the harness driven by a virtual pad and OBS), 4.y-ingame2 started
2026-10-10 (the remaining in-game checks: collars beyond the stock set load on PES 21,
the sock table seen on PES 21, the DX10 header crashes PES 21's BC3 portrait reader),
4.y-fix16 done 2026-10-10 (S9's second Astra round: the deep pass reads a player's roles as
planning does, drops a beaten Common model with its winner, leaves an xml's unnamed files and
Fox's unsearched Common `.mtl` files unread, skips a refs export's kits, logo, portraits and
collars; `common_model_beaten_dropped`), 4.y-sub done 2026-10-11
(a subfolder of a player folder is a player folder of its own; (a) done 2026-10-10: a
subfolder's files take their roles from their names at any depth, the reserved subfolders
gone, a refs export's AET layout refused per folder; (b1) done 2026-10-10: a subfolder's
textures at their paths, a texture name resolving nearest first, each folder its own stem
namespace, the singletons root-only; (b2) done 2026-10-10: the pre-Fox face packs and
lists a subfolder's files at their paths, `./sub/name` a checked local reference; (b3a)
done 2026-10-10: a texture named by its path below its file's folder resolves there, a
Fox subfolder texture before `sourceimages`; (b3b1) done 2026-10-10: links at any depth, a
`.common` model or material link standing for the `Common/` file at its path; (b3b2)
done 2026-10-10: a texture link below a subfolder counts in its own folder, its texture
packed once at its path in the team's Common output; (b3b3) done 2026-10-10: a player's
own texture or texture link wins over a combined folder's, `shared_texture_overridden`;
(b3b4a) done 2026-10-10: on PES 15-17 a `Common/` `.mtl` resolves its textures from its
own `Common/` directory; (b3b4b) done 2026-10-10: on Fox a linked `Common/` subfolder
model's textures travel with it and resolve beside it; (b3b4c) done 2026-10-11: S18.A3's
fixes, a linked Common file bringing the textures above it and a player `.mtl` over a
Common model falling back beside the model; (b3c1) done 2026-10-11: a shared folder's
subfolders follow the player folder's rules; (b3b4d) done 2026-10-11: a texture path
into a `Common/` subfolder is checked on both engines; (b3b4e) deferred, pre-Fox;
(b3c2a) done 2026-10-11: a shared folder's `.common` links admitted and checked, its
texture links counting on Fox; (b3c2b) done 2026-10-11: on Fox its model and
material-file links count too; (b4a) done 2026-10-11: a kit set's `kitF` fallback, written
once as `kitN` on Fox; (b4b1) done 2026-10-11: a `.common` texture link naming a kit set
finds it; (b4b2a) done 2026-10-11: a kit-token folder set completed, `Common/kit1/` and
`Common/kit2/` as one; (b4b2b1) done 2026-10-11: a `kitN/` texture path finds its set, a
`kitF` one nothing; (b4b2b2) done 2026-10-11: a `.common` link to a kit set falls only
with its last member; (c) and (d) found done by earlier slices; the step's verify run
on a new-layout copy of the HAHA census export; done 2026-10-11), 4.y-fix20
done 2026-10-10 (S17's Astra round: the hidden-model drop decided once by the deep pass,
which now takes a folder's own file off on `DropFile`; a package's names folded; the hand
split's tests by faces), 4.y-fix21 done 2026-10-10 (the S18, S7 and S17 rounds' small fixes:
TC-MOD-64's meshes told apart, a dropped shared link no link, a textures-only folder
reported, a `.model` collar's material losses skipped), 4.y-fix22 done 2026-10-10 (the S13
and S15 rounds' rework; S12, S14, S15 and S17 closed), 4.y-fix23 done 2026-10-11 (a shared
boots or gloves folder's model link named as its models are; its pre-Fox item an issue),
4.y-mattable done 2026-10-11 (the material table kept against the wiki on a census of real
FMDL and `.mtl` files; three family defaults left for Phase 7), 4.y-simp done 2026-10-11 (the
simplification pass: a texture conflict leaves the player folder out, the salvage machinery
gone, net about -270 lines), 4.y-scen done 2026-10-11 (every CLI test cites its scenario;
363 scenarios, all proven), 4.y-kitwarn done 2026-10-10 (a sleeve or fit option the
shirt model ignores is `kit_config_option_ignored`, the config checked as emitted, the FPC
values applied first on an FPC team), 4.y-retag done 2026-10-10 (no retype needed; 76 long subjects cut, history rewritten
and force-pushed), 4.y-fix12 done 2026-10-10
(a `.model` collar converts with the templates' `uniform.mtl` on both engines, the
pre-check moving a PES 15-17 one onto the version's skeleton; TC-CMN-11, TC-CMN-19), 4.y-fix17
done 2026-10-10 (the marker's combined folders in `held_stems`, `texture_unreadable` and
`texture_codec_unsupported` from the deep pass's probe, the real stock collar fixture), 4.y-fix18 done 2026-10-10 (the converted Fox collar takes the
stock collars' material set by name, its meshes two-sided: PES 21 drew nothing for the
converter's and draws it now, checked in game), 4.y-fix13 done 2026-10-10 (a hidden Fox mesh left out of a
`.model`; a model with every mesh hidden left out whole with the Info
`model_hidden_dropped`, after the lead's scan of every FMDL on the machine), 4.y-fix15 done 2026-10-10 (a `.face` link to a `Faces/` folder holding textures alone
gives no face: his own face files `face_file_not_used`, the blank face with the bundled
diff; its items (1) and (3) had landed with 4.y-fix17 and 4.y-fix16), 4.y-fix19 done
2026-10-10 (S9's third Astra round: every direct Common `.mtl` read once a link's search
lands in `Common/`; a non-model file in `Collars/` gets no collar finding), 4.y-fix14 done 2026-10-10 (on PES 15-17 every directory of `Common/` is packed at its
own path and a member's `face.xml` names a subfolder's file, a refs export's `Common/` lays
over the referee template's `common/999/` tree, which a referee's `face.xml` may name, and
a referee folder holding `fpc_off` lists the refkit body; on PES 18-21 a subfolder's file
is `file_not_used`),
then the `duck` reviews (started 2026-10-09; Astra's five-hour quota stopped the first two mid-review, retried from 14:50), 4.c-threshold done 2026-10-09 (0.7 kept), 4.z-rewrite
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
  shares the blind spots the review exists to catch). Astra has two quotas per ChatGPT Plus
  account, a five-hour one that one whole-module review takes most of (S9: 78%) and a weekly
  one of about eight such reviews (S9 to S11, 2026-10-09: 7-12% each): when a
  call stops for quota, the review goes on the queue below (commit range, plan sections), the
  lead retries once an hour, and once a call works the queue runs until the next stop. Work
  continues past a queued review, but no phase closes with one outstanding (two exceptions:
  Phase 3 closed with its reviews queued, by the maintainer's directive of 2026-10-02, and
  Phase 4 closes on SWE-2 reviews alone, by the maintainer's directive of 2026-10-09: Astra's
  windows were spent, so every Phase 4 surface's remaining Astra rounds are queued below as a
  re-audit, run as the windows clear, and each SWE-2 loop still runs in full). The
  backlog below (S7-S16 of `.tmp/lead/review_queue.md`, plus 4.14's (a) and (b) and 4.17's
  (c) and (d)) runs as the first half of Phase 4's converge, on the finished code grouped by
  module, by the maintainer's decision of 2026-10-09: the remaining slices (4.17g, 4.19d,
  4.20, 4.27) rewrite parts of what those reviews would read, and converge reviews the
  crate anyway.
- **Queue: empty** (2026-10-11). The Phase 4 Astra re-audit (directive above; briefs in
  `.tmp/duck_brief_S*.md`, rulings in `.tmp/4_y/duck_rulings.md`) is done: S7 to S15 and S17
  (4.14's (a) and (b) and 4.17's (c) and (d), the conversion pair) ended under five accepted
  and are closed (status above); S16, the Phase 3 crates' second whole-crate mutation runs,
  is no review. The items below are the old per-step queue, each covered by the surface
  `.tmp/lead/review_queue.md` maps it to (S1-S16), kept for their pointers. 3.7 (b): `crates/libs/pipeline` from its first commit, against `libs/pipeline.md`
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
`team_compiler/pipeline.md` "Run driver shapes", `aesthetics_export/object_model.md`,
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
  `native_field_dropped (field=tags)`; the stock cap's tag is `Captainmark`. Answered at
  4.y-ingame2 (2026-10-10): PES 17 draws the tagged mesh on a non-captain as it draws the
  untagged one, in Edit mode and in a match, so the IR does not need the field (DECISIONS
  2026-10-10). Also for converge (maintainer, 2026-10-08):
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
- [x] 4.y-fix7 **The deep pass checks what `compile` reads** (S9's rework: Astra's round
  S9.A1 and the SWE-2 loop after it, rulings in `.tmp/4_y/duck_rulings.md`). (1) A kit
  texture planning drops (`drop_unused_kit_textures`: the other engine's map, a `kit_*`
  stem outside the seven) is not checked: one predicate beside `KIT_TEXTURE_STEMS` for
  both. (2) A `Common/` model another representation of its stem beats is not checked
  (`selected_common_model`'s rule). (3) `KeptCommon` holds the `Common/` files the report
  keeps, `pass_through` included (the pass takes the setting). (4) A user `face.xml`'s
  pairing of a Common model with an entry's `.mtl` is compared and reported on the
  folder. (5) A file is read and paired only when its role is one `compile` reads: not a
  left-out kit variant (`PlayerFile::LeftOutKitVariant`), not a model with no role (a
  `.model` in a player's `common/`; on pre-Fox `pairings` paired every `.model`). (6) A
  `Common/` file below a subfolder is neither checked nor looked up. (7) On PES 15-17 a
  `face_diff.bin` beside a member's `face.xml` holding a `<dif>` is not checked. (8)
  `fmdl::check::Subject::Model` removed (no producer). (9) A second slice moves the
  pairing code into `deep/pairings.rs` unchanged (`deep/mod.rs` past 1000 lines).
  Plan: `pipeline.md` deep pass bullet and "Kit-dependent assets"; DECISIONS 2026-10-09.
  TC-KIT-31, TC-CMN-14, TC-CMN-15, TC-CHK-09, TC-XML-12, TC-XML-13, TC-MOD-62,
  TC-MOD-63. Crates: tc, fmdl
  → verify: the eight scenarios proven, red first.
  Slice (a) done 2026-10-09 (Opus 5.5, landed first time): items (1) to (8).
  `roles::emits_kit_texture` and `roles::is_selected_common_model` shared by planning and
  the pass; `content_findings` takes `pass_through`, `drops_file` is the report's rule;
  `folder_findings`' `unread` is "no role or a left-out variant" (a beaten model has no
  role), the pre-Fox `pairings` pair a `.model` only as `PreFoxModel` or `PreFoxPart`, an
  xml's Common pairing is reported after the folder's files (`named_on_folder`, one naming
  rule for model and `.mtl`), and `folder_findings` returns whether a parsed xml holds a
  `<dif>`, which skips the pre-Fox `.bin` check. TC-CMN-05's test no longer expects the
  left-out `pants_kit2.fmdl`'s weights Info (the change asked for). Two premises found
  incomplete, not built: a marked pre-Fox folder's left-out variant is still read (issue
  below), and a `Common/` model a Common glTF beats is still checked (no output changes:
  planning drops the glTF's linking players). Gates green (acceptance 295 of 295);
  `mutants-diff 944fb65`: 26 mutants, 23 caught, 3 unviable, 0 missed.
  Slice (b) done 2026-10-10 (Opus 5.5, landed first time): item (9), the pairing block
  (`Pairing`, `pairings`, `pairs_with_mtl`, `links_common_fmdl`, `searched_common_mtls`,
  `used_names`, `material_finding`, `undefined_materials`) moved into `deep/pairings.rs`
  unchanged, every line equal to its original in order (the lead's check), the
  visibility words and both files' imports the only edits; `listed_materials`, the xml
  half, stays with the xml readers; `mod.rs` 2918 → 2658 lines (1368 before its tests,
  still past the mark: the design-health pass's). Test counts equal before and after;
  gates green (`.tmp/4_y/gates_fix7b.log`); mutants 31/0/4
  (`.tmp/4_y/mutants_fix7b.log`, the moved code's).

- [x] 4.y-fix8 **Kit colors and kit configs: S10's and S11's rework** (Astra's rounds
  S10.A1 and S11.A1 and the SWE-2 loops after them, rulings in
  `.tmp/4_y/duck_rulings.md`). (1) The deep pass does not check a referee export's root
  `colors.txt`, which nothing reads (`deep/mod.rs`). (2) `colors_txt` unit cases for a
  leading `+` and a tab (refused) and leading zeros (read). (3) `bins::kit_number` renamed
  `uni_color_kit_number` (`kit_variants::kit_number` is the `kitN` numbering). (4) An
  absent slot's installed config re-encoded for the FPC values or the collar reports
  `kit_config_version_clamped` on the export naming the slot, from `kit_config::validate`'s
  two range findings on the decoded config. (5) `kit_config_collar_unpatched` when the
  collar cannot be worn and no FPC finding named the slot (`edit_absent_slot`'s comment
  corrected). (6) TC-CMN-08's collar-117 half compiled, not checked. (7) A model below a
  `Collars/` subfolder is neither name-checked nor claimed (`export_collar`,
  `collar_findings`). (8) The help topic names the new code. Plan: `pipeline.md`
  "Collars" and "Bins accumulation", `messages.md` (`kit_config_invalid` reworded to
  what `kit_config` refuses, `kit_config_version_clamped`, `kit_config_collar_unpatched`);
  DECISIONS 2026-10-09. TC-BIN-23, TC-BIN-24, TC-CMN-08, TC-CMN-16, TC-REF-14. Crates: tc,
  aesthetics_export
  → verify: the five scenarios proven, red first; the `colors_txt` cases pass.
  Done 2026-10-09 (Opus 5.5, landed first time; one brief error caught: a config encoded for
  PES 21 cannot hold Name Y 40, so the test sets the bits by hand). `edit_config` collects
  what the encode clamps (`deep::version_clamped`, shared with `kit_config_findings`) and
  `edit_absent_slot` reports it naming the slot; `wear_collar` returns `Option<bool>`;
  `is_direct_common_file` became `is_direct_root_folder_file`. On PES 15-17 with no install a
  Midcup export with a collar reports `kit_config_collar_unpatched` for each unexported kit the
  bundled record lists (seven for `/co/`), as `kit_config_fpc_unpatched` does: kept. Lead
  fixes: the help's clamp paragraph names the absent slots' clamp; three doc comments
  rewrapped. Gates green but acceptance's two fix9 scenarios (TC-XML-14, TC-XML-15, opened
  meanwhile); `mutants-diff 0bb67f8`: 86 mutants, 66 caught, 20 unviable, 0 missed.

- [x] 4.y-fix9 **A shared face folder's `face.xml` rules its linking faces** (the
  maintainer's answer, 2026-10-09). On PES 15-17 a `Faces/` folder's `face.xml` is checked by
  the deep pass as a member's own in the shared folder's pass, and is the xml of each
  linking player's face, his models it does not name appended as generated entries before
  the `face_neck` dummy and the `<dif>`; a player holding a `face.xml` and linking a
  `Faces/` folder is `xml_shared_face_conflict` (E, `DropFolder`, not pass-through-eligible);
  `xml_ignored_shared` stays for `Boots/` and `Gloves/` folders only. The help topic says
  so in the same diff. Plan: `messages.md` "User-supplied `face.xml`" (the paragraph
  starting "A `Faces/` folder's `face.xml`"), `xml_ignored_shared`,
  `xml_shared_face_conflict`; DECISIONS 2026-10-09. TC-XML-14, TC-XML-15 (TC-XML-10
  withdrawn: its test changes). Crates: tc
  → verify: the two scenarios proven, red first.
  Done 2026-10-09 (Opus 5.5, landed first time; one contradiction accepted: the conflict is
  found in the deep pass's player loop, `deep::shared_face_conflict`, not at planning,
  since `check` never plans and a planning finding comes too late to drop the folder).
  `ModelFolder::face_xml` (a player's own `face.xml` or his linked `Faces/` folder's)
  replaces `own_face_xml`; `FolderModels::is_shared_boots_or_gloves` scopes
  `xml_ignored_shared`; `prefox_face.rs` `names_model` decides which models the xml names
  and `last_dif` which `<dif>` it writes. A deep-pass test that linked a face beside the
  player's own xml now expects the conflict (the change asked for). Gates green
  (acceptance 300 of 302: TC-MOD-64 and TC-CMN-17 are 4.y-fix13's and 4.y-fix10's);
  `mutants-diff ca18f43`: 52 mutants, 45 caught, 6 unviable, 1 missed (`names_model`'s
  `kitN` clause untested: carried into 4.y-fix10, whose sets it decides).

- [x] 4.y-fix10 **A pre-Fox per-kit model set is completed against the kit numbers** (the
  maintainer's answer, 2026-10-09). On PES 15-17 a model set a face's `face.xml` lists,
  generated or the member's own, gets for each kit number the export defines that it lacks
  the files the lowest variant's entries name packed again under that number's spelling
  (its models, its `.mtl` when that carries the token), with `kit_variant_missing`
  (context `model`), as the textures task completes a texture set. Plan: `pipeline.md`
  "Kit-dependent assets" (the sentence starting "The set is completed as a texture set
  is"), `model_format.md` "Per-kit models", `messages.md` `kit_variant_missing` and
  "User-supplied `face.xml`"; DECISIONS 2026-10-09. TC-CMN-17. Crates: tc
  → verify: TC-CMN-17 proven, red first; a member's own xml naming a set is completed too,
  and none of the set's variants gets a generated entry beside it (`names_model`'s `kitN`
  clause, 4.y-fix9's survivor: `delete !` at `prefox_face.rs` ~920 must be caught).
  Done 2026-10-10 (sidekick, two runs: the first was killed with the session, the
  second finished from its staged work): the export's kit numbers ride on
  `TaskKind::Models` to the pre-Fox face; `set_completions` copies the lowest variant's
  model, hands and own `.mtl` under each missing number's spelling after the packing
  loop, and `XmlFace::complete_set` does the same for a `kitN` reference of a member's
  xml (the `.mtl` set silently). A lone variant stays an ordinary model (plan sentence
  added). TC-CMN-17 proven; six tests, red first; the fix9 survivor's scenario had to
  give the player two higher variants to kill the mutant (the sidekick's contradiction,
  accepted). Acceptance 301 of 307. `mutants-diff f644ec1`: 22 mutants, 18 caught, 2 unviable, 2 missed, both
  tests to add (fix11 items 13 and 14): `set_completions`' guard that an unlisted
  variant belongs to the set at hand (two sets in one face), and `complete_set`'s
  finding on a `.model` reference whose set has one shared `.mtl`.
  Files: `plan/mod.rs`, `plan/item_rows.rs`, `processing/mod.rs`,
  `processing/prefox_face.rs`, `messages.rs`, `help/01_command_line.md`,
  `tests/cli/prefox_kit_variants.rs`, `prefox_hand_split.rs`, `prefox_user_xml.rs`.

- [x] 4.y-fix11 **S7's, S8's, S12's and S13's SWE-2 rework** (rulings S7.1, S8.1, S12.1,
  S13.1 in `.tmp/4_y/duck_rulings.md`). (1) A shared `Boots/` or `Gloves/` folder gives no
  face: its face files are `face_file_not_used` (`roles.rs` `FolderModels::read`). (2) A
  combined shared source's `.model` is searched with no `Common/` files, as the deep pass
  does (`processing/model.rs` ~157-176: a kept `x.mtl.common` there panics today). (3) A
  shared folder's roles take only the files directly in it (`of_shared`): a kept file
  below its subfolder compiles today. (4) A refs export gives `complete_kit_variants` no
  kit numbers (`plan/mod.rs` ~1004). (5) `FaceUse::Used::combined`'s doc reworded (team
  players combine too). (6) A `kitN` texture stem is looked up in the installed CPKs with
  `has_variant_among` over `InstalledPaths::common_texture_stems` (`processing/model.rs`
  `texture_supply`), `None` staying `Unknown`. (7) A WESYS-wrapped DDS portrait goes out
  unwrapped on PES 18-21 (`texture.rs` `portrait`). (8) `common_places` and
  `point_texture`'s Common stems take the folder's `.common` texture links too. (9) A kit
  variant set is formed over every file the part is built from, not per directory
  (`kit_variants.rs` `model_variant_sets`). (10) Plan only: layout conversion keeps BC1
  and BC3 blocks, a BC7 source encoded whole (no code change). Plan: `pipeline.md`
  "Kit-dependent assets", "Layout conversion", `messages.md` `face_file_not_used`,
  `object_model.md` "File-type allowlist"; DECISIONS 2026-10-09. Astra's rounds on S12
  and S13 (rulings S12.A1, S13.A1, one concern each, both accepted): (11) TC-KIT-19's
  Fox-to-pre-Fox half asserts the re-laid stripe centres at the inverse table's
  positions, as TC-KIT-18 does, not only that nothing outside the bands moved (test
  only). (12) `texture.rs` `texture_role` reads the stem without its kit token
  (`without_kit_token`), so a per-kit normal map `pants_nrm_kit1` (`model_format.md`
  "Per-kit models": `{m}{suffix}_kitN`) is encoded as a normal map on PES 18-21, not
  as color. Crates: tc
  → verify: a test per item (1)-(4), (6)-(9), (11), (12), red first; S7 gets a second
  SWE-2 round on the rework diff, and S12 and S13 their SWE-2 rounds on it.
  Done 2026-10-10 (sidekick, one run): every item landed with its test, red first;
  item (9) as the sidekick's two contradictions, both accepted: a part's sets span all
  its sources only where the player's own folder leaves out variants (a pre-Fox
  referee's combined boots are an output of their own, and spanning them emptied his
  boots folder in a test), and a set is reported on the player only when it holds a
  file of his own (the literal rule reported a shared folder's set once per linking
  player, which an existing test forbids); two files of one kit number in two
  directories stay "neither used nor left out". Brief errors the sidekick caught: (1)'s
  Fox case and (2)'s scenario could not go red (a face-named `crocs.fmdl` and a
  `materials.mtl` beside a name-matched link did), (13)'s lone `socks_kit1` is never
  completed (`socks_kit3` added). (7): `processing::wrap_dds` re-wraps the portrait on
  PES 15-17 when `dds_compression` is on. (8) is reachable only through a texture link
  an earlier installed CPK satisfies (a midcup export). Gates green, acceptance 301 of
  312 (the eleven unproven are open steps'). `mutants-diff 069703f`: 43 mutants, 32
  caught, 10 unviable, 1 missed (`part_sources(index + 1)` in `ModelFolder::roles`:
  fix16 item (9)). Follow-ups to fix16: the deep pass's per-folder roles (6), the
  `admitted` double report (7), the pre-Fox siblings of (2) (8). The TC-KIT-19 margin
  is an issue below.
  Files: `src/deep/mod.rs`, `src/kit_variants.rs`, `src/plan/mod.rs`, `src/plan/roles.rs`,
  `src/processing/model.rs`, `src/processing/texture.rs`, `tests/cli/common_links.rs`,
  `kit_layout.rs`, `models.rs`, `prefox_kit_variants.rs`, `prefox_kits.rs`,
  `prefox_user_xml.rs`, `referees.rs`, `textures.rs`.

- [x] 4.y-fix12 **A `.model` collar converts with the templates' `uniform.mtl`** (the
  maintainer's answer, 2026-10-09). Lead first (a template is correctness-critical): PES 17's
  stock `uniform.mtl` extracted from its install's data CPK into `resources/templates/`,
  its provenance recorded as the other templates' is. Then the sidekick: a `.model` collar
  compiled for PES 18-21 converts with it as its `.mtl` (no more `model_conversion_failed`
  at planning), and a `.model` collar the same-engine pre-check flags on PES 15-16 is posed
  with it as its set (the 4.17 "Open for converge" item: a collar posed off the target's
  skeleton was packed as it is). Plan: `pipeline.md` "Collars" and step 3 "Format
  conversion" (the `.model` collar sentence), `messages.md` `model_conversion_failed`;
  DECISIONS 2026-10-09. TC-CMN-11 (rewritten: its test changes). Crates: tc, resources
  → verify: TC-CMN-11 proven, red first; the flagged stock PES 17 collar posed on PES 15.
  Also (lead's finding, 2026-10-09): a collar converted for PES 15-16 names `uni_shirts`
  alone, PES 15's and 16's `uniform.mtl` defining no `uni_collar` (`pipeline.md`
  "Collars"); TC-CMN-19.
  Lead part done 2026-10-09: `resources/templates/uniform.mtl` is PES 17's `dt32_win.cpk`
  copy, provenance in `resources/templates/README.md`, `-text` in `.gitattributes`. The
  cup's `4cc_04_fpc.cpk` overrides the file in game on PES 15-17, differing only in
  `skin_limb`; its PES 15-16 copy has no `uni_collar` either, so the PES 15-16 rule holds
  on the cup's installs.
  Done 2026-10-10 (sidekick, one run; brief `.tmp/4_y/brief_fix12.md`, report
  `.tmp/4_y/sk_fix12_report.md`): `Templates::uniform_mtl()` (the fifteenth overridable
  resource); `CollarFile::NoMaterialSet` gone, a `.model` collar `Compiled` on both engines;
  the collar task runs a PES 18-21 `.model` through `model_for_fox` with the template as
  its `.mtl` and `ConvertedMaterials::StockCollar` (`PreFoxMaterials` renamed, now a
  parameter of `model_for_fox` and `fox_written` too), which drops every sampler of the
  FMDL (`collar_samplers_dropped`: the game's own collars bind only `Pattern_Tex_LIN`, a
  sampler the converter never writes, so the brief's "replace with the stock path" branch
  could never fire and was not built) and reports no loss about a material; the skeleton
  the conversion may write is dropped with no finding; a PES 15-17 `.model` collar goes
  through `model_for_pre_fox` with the template as its set, so one the pre-check flags is
  moved onto the version's skeleton; `stock_collar_materials(model, version)` names
  `uni_shirts` alone on PES 15-16. TC-CMN-11 rewritten and proven, TC-CMN-19 proven, the
  pre-check test on PES 15 against PES 17 byte for byte. The brief's fixture was wrong:
  `konami_collar_052.wesys.model` is Konami's `modD_shirt_tight_in_collar_052.model`, one
  material `modD_phone`, which the template does not define, so the tests convert it with
  that material renamed `uni_shirts` (`uniform_collar_model()`); a real stock collar as a
  fixture (the lead's, with its provenance: PES 17's are not in `dt32_win.cpk`, their CPK
  still to find) goes to the next rework brief. New on PES 15: a `.model` collar the
  pre-check flags whose material the template lacks is `model_conversion_failed` (it was
  packed); on PES 16-17 such a collar still packs. Open for 4.y-ingame2: the converted Fox
  collar keeps the converter's `fox3ddf_blin` with no sampler, where the stock PES 21
  collars use `pes_3ddf_collar` and `pes_3ddf_shirt_nb` with `Pattern_Tex_LIN` (to see
  on PES 21); and real PES 17 collars list `uni_shirts` first, `uni_collar` second, mesh
  0 on `uni_shirts`, against the plan's "first material becomes `uni_collar`". Files:
  `templates.rs`, `plan/collars.rs`, `plan/roles.rs`, `processing/conversion.rs`,
  `processing/mod.rs`, `processing/model.rs`, the three `prefox_*.rs` imports,
  `tests/cli/collars.rs`, `help/01_command_line.md`; `pipeline.md` "Collars" and the
  templates README's sampler sentence (lead). Gates green (`.tmp/4_y/gates_fix12_lead.log`,
  acceptance 302 of 312); mutants 23 caught, 0 missed, 10 unviable
  (`.tmp/4_y/mutants_fix12.log`).

- [x] 4.y-fix13 **A hidden Fox mesh is left out of a `.model` export; a model with every
  mesh hidden is left out whole** (the maintainer's answer, 2026-10-09; the lead's FMDL
  scan, 2026-10-10: `.tmp/4_y/fmdl_flags/flags.tsv`, 61 hidden meshes of 35,251 in
  7,315 files, 38 files all hidden: blank-head orals, one-sided gloves, stadium shadow
  models). `model_convert`'s pre-Fox export skips a mesh whose `invisible` flag is set,
  with no finding (`mesh_flags_dropped` keeps `no_shadow_cast` alone); an export left
  with no mesh is `ConvertError::EveryMeshHidden`, which the compiler's
  `fmdl_for_pre_fox` turns into "nothing written" with the Info `model_hidden_dropped`
  at each of its four callers (face, shared boots and gloves, Common, collar). Plan:
  `model_conversion/ir.md` (the bullet starting "A hidden Fox mesh"), `messages.md`
  `mesh_flags_dropped` and `model_hidden_dropped`; DECISIONS 2026-10-09 and 2026-10-10.
  TC-MOD-64, TC-MOD-69. Crates: model_convert, tc (its consumers' tests: `just bindings`
  too) → verify: both scenarios proven, red first; unit tests of the export's skip and
  of the all-hidden error; the legacy oral parity kept with the bit cleared.
  Done 2026-10-10 (Opus 5.5, one run): `ir_to_model` records which materials hide their
  meshes, files `native_field_dropped` for `no_shadow_cast` alone, skips a hidden mesh
  whole (its own findings too), takes the bounds from the written meshes and returns
  `ConvertError::EveryMeshHidden` when none is; `pre_fox_written` takes a `Converted`, so
  `fmdl_for_pre_fox` runs `convert` itself and returns `Option`, `None` with the Info
  `model_hidden_dropped`; each caller leaves the model out (the face neither packs nor
  lists it, its packed names still replacing a linked shared face's; a boots part not
  merged, `model_merged` counting the parts written; a glove side with no entry and no
  file; a Common FMDL with no `.model` and no `.mtl`; a collar with no entry). Tests:
  TC-MOD-64 (the tracer's glove as `face_high.fmdl` with a copy of its mesh hidden: one
  mesh written, no `invisible` Warning, the plain run's findings plus the import's
  `material_split_by_flags` for the shared material, which stays, the import having split
  it), TC-MOD-69 (the blank head's `addon_oral.fmdl` beside a `.model` face: the face's
  three files alone, the xml naming the face model alone, the Info naming `oral.fmdl`),
  three unit tests in `model_convert` and the legacy oral parity with the bit cleared on
  the parsed model. Contradictions, four, all brief errors accepted: the tracer has no
  `face_high.fmdl` (`clean_model()` used); the split Info (ruled above); every packed
  pre-Fox face model is named `oral_<stem>_win32.model`, so "no entry holding `oral`"
  could not hold (TC-MOD-69's text aligned to the `.model` face `write_slot_05_face`
  writes); the parity test clears the bit on the parsed model. Edge cases it left as they
  are, filed as issues below: an all-hidden boots folder, an all-hidden gloves folder, a
  `.fmdl.common` link to an all-hidden Common FMDL. Files: `model_convert` `formats/mod.rs`,
  `formats/pes_model/export.rs` and `tests.rs`, `convert.rs`, `loss.rs`; `team_compiler`
  `messages.rs`, `processing/conversion.rs`, `mod.rs`, `prefox_face.rs`,
  `prefox_shared.rs`, `prefox_common.rs`, `tests/cli/conversion.rs`, `prefox_faces.rs`;
  README TC-MOD-69 (lead). Gates green (`.tmp/4_y/gates_fix13.log`, acceptance 304 of
  313), `just bindings` green (37 fixtures); mutants 13 caught, 0 missed, 3 unviable; the remote half peaked at the 9G cap with no build killed (`.tmp/4_y/mutants_fix13.log`).

- [x] 4.y-fix14 **`Common/` subfolders, the referee body** (the maintainer's answers,
  2026-10-09). Three slices, (a) split in two at briefing: (a1) the subfolders packed at
  their own path and named by a member's xml, (a2) the refs export's `Common/` over the
  referee template's `common/999/` tree and the tree as an xml source, (b) `fpc_off`.
  (a) On PES 15-17 a `Common/` subfolder is packed under the team's
  Common output at its own path, processed as `Common/` is; a `face.xml` Common path naming
  `<subfolder>/<name>` resolves there (no more `xml_path_unchecked`), in a refs export the
  referee template's `common/999/` tree counting too; a refs export's Common files lay over
  the template's of the same path; the allowlist admits model content anywhere in `Common/`;
  the deep pass checks a subfolder's files on PES 15-17 (they are read). (b) A referee
  folder's `fpc_off` on PES 15-17 appends the refkit body's ten entries (`blue_port.md`'s
  table) to his face's xml, leaving out what the xml names and the boots or hands he has
  his own of. Plan: `pipeline.md` "Common", `messages.md` "User-supplied `face.xml`" (the
  Common reference bullet), `blue_port.md` "The referee body",
  `aesthetics_export/object_model.md` allowlist; DECISIONS 2026-10-09. TC-CMN-18, TC-REF-15
  (a); TC-REF-16, TC-REF-17 (b). Crates: tc, aesthetics_export → verify: the four scenarios
  proven, red first; the census re-run over the six refs exports holding `Common/refkit/`:
  no `common_file_disallowed` and no `xml_path_unchecked` for it.
  (a1) done 2026-10-10 (Opus 5.5, one run and two rework rounds): the allowlist admits
  model content anywhere in `Common/` (`check_common`, the stem check over the direct
  textures alone); `roles::is_read_common_file` (every directory on PES 15-17, `Common/`
  itself on Fox) and `directory_stem` (per-stem selection within a directory) replace the
  direct-file guards in planning, the deep pass and `validation.rs` (a Fox subfolder's file
  `file_not_used`); `plan_run` plans one `CommonTextures` and one `CommonModels` task per
  read directory (`common_directories`), the environment map flag for `Common/` itself
  when any read directory converts a metal FMDL (the rework round: a subfolder's converted
  material names the root `env.dds`); outputs take `paths::common_subpath` (`texture.rs`
  `common_texture_entries`, `prefox_common.rs`, whose `skeleton_beside` pairs a subfolder
  FMDL's `.skl` in its directory); a `Reference::Common` holds the path relative to
  `Common/` (`user_face_xml.rs` `reference`, `resolve`; `XmlFace::written_path` keeps the
  subfolder; a direct-only guard in Fox `point_texture`, `deep/materials.rs` `supply` and
  `prefox_shared`'s Common textures, which a link alone reaches); `common_mtl_findings`
  resolves a `.mtl`'s textures in its own directory; `named_on_folder` names a subfolder's
  file by its export path. A subfolder's glTF is selected within its directory and dropped
  with `model_gltf_unsupported` (one rule). Tests: TC-CMN-18 (`prefox_user_xml.rs`: the
  refkit thigh and `.mtl` packed under `refkit/`, the xml entry with the team ID), TC-CMN-13
  and TC-CMN-15 rewritten (`file_not_used` on PES 21, `model_broken` on PES 17), the nested
  texture test asserting `sub/cloth.dds`, unit tests for `reference`, `resolve`,
  `common_directories`, `common_subpath`, `named_on_folder`, the flag. Contradictions ruled:
  `file_not_used` is a Warning (the README said Info: corrected); the flag (accepted, the
  rework); an empty `<rest>` is `Unchecked` rather than `xml_model_not_found` (accepted: the
  plan's catch-all). Not run: the census over the six refs exports (their `refkit.mtl`'s
  textures and TC-REF-15 are (a2)'s; run it after (a2)). Files: `aesthetics_export`
  `validate/folders.rs`, `tests.rs`; `team_compiler` `deep/materials.rs`, `deep/mod.rs`,
  `paths.rs`, `plan/mod.rs`, `plan/roles.rs`, `processing/mod.rs`, `model.rs`,
  `prefox_common.rs`, `prefox_face.rs`, `prefox_shared.rs`, `texture.rs`,
  `user_face_xml.rs`, `validation.rs`, `tests/cli/common_links.rs`, `prefox_faces.rs`,
  `prefox_user_xml.rs`; README TC-CMN-13/15 (lead). Gates green
  (`.tmp/4_y/gates_fix14a1.log`, acceptance 305 of 313); mutants 96/0/16
  (`.tmp/4_y/mutants_fix14a1.log`; the second rework round: the two survivors were the
  direct-texture filters of the deep pass's Common stem set and of the shared package's
  Common texture map, each now with a test of a `.mtl` path naming a subfolder texture's
  or a direct model's stem).
  (a2) done 2026-10-10 (Opus 5.5, one run, no rework): `output/writer.rs` `Referees`
  keeps the path of every refs-task entry that went in (`entry_paths`, noted in `add`, so
  sideload mode too) and `finish_referees` leaves the tree's file at such a path out with
  no finding, compared exactly as the CPK's duplicate check compares (the marker's path and
  an override's `duplicate_path` as before); `templates::referee_common_paths(engine)`, a
  free function rather than the brief's `Templates` method (the sidekick's contradiction,
  accepted: the deep pass runs in `check`, which reads no templates, and an override
  replaces a tree file's bytes, never adds a path), gives the tree's `common/999/` files by
  their path relative to it, folded (16 on PES 15-17, none on Fox); `FaceFiles` carries
  them as `template_common` and `reference_finding` admits a Common reference `resolve`
  misses when the list holds its path and the file is of the referenced kind; the deep pass
  builds the list once per refs export (`KeptCommon::template_paths`). The face task passes
  an empty list (the lead's choice over the sidekick's refs-only branch, which nothing
  read: `written_path` and `written_material` write a Common reference by its path,
  packing nothing, so a template file's entry is written as the member spelled it). Tests:
  TC-REF-15 (`referees.rs`: the export's `refkit/texture.dds`, converted for PES 17, in the
  refs CPK with the tree's fifteen other refkit files beside it, no `duplicate_path`), a
  referee's xml naming the tree's `oral_arm_*.model` and `refkit.mtl` the export lacks
  (no `xml_model_not_found`, the entry as written with `999`, no arm file in the face
  CPK), writer and `templates.rs` and `user_face_xml.rs` unit tests. Files:
  `team_compiler` `deep/mod.rs`, `output/writer.rs`, `processing/prefox_face.rs`,
  `templates.rs`, `user_face_xml.rs`, `tests/cli/referees.rs`. Gates green
  (`.tmp/4_y/gates_fix14a2.log`, acceptance 306 of 313); mutants 16/0/5
  (`.tmp/4_y/mutants_fix14a2.log`). The census re-run over the twelve refs exports
  (`.tmp/4_y/census/roots_refs.txt`, `logs_17_refs/`, PES 17 `check` and `compile`): no
  `common_file_disallowed`, `xml_path_unchecked` or `duplicate_path` anywhere; the six
  that failed the allowlist now fail on `model_material_undefined` alone, a real defect
  (one referee's face names `head_phong`, its `.mtl` defines `head`; one's gloves name
  `skin_limb`, undefined too); their `refkit.mtl` is read (`mtl_state_*` Infos) and a
  stray `Common/refkit/face.xml` is `file_not_used`; a `--no-deploy` refs CPK holds each
  `common/999/refkit/` file once beside the referees' own subfolders.
  (b) done 2026-10-10 (Opus 5.5, one run, no rework): `ModelFolder::refkit_body`, set by
  `plan_run` for a refs export's folder holding `fpc_off` on PES 15-17 alone (a team
  player's body is the kit's; the Fox refkit is not made); the face task (`prefox_face.rs`
  `refkit_body`, `refkit_body_entries`, the `REFKIT_BODY` table in the plan's order) appends
  the template's ten entries after every model of his, generated or his xml's, before the
  `face_neck` dummy, naming `model/character/uniform/common/999/refkit/oral_<name>_*.model`
  with `refkit.mtl`, packing nothing; left out: a model the face lists already (a refkit
  entry he wrote by hand counts once and is not his own boots or gloves), the boots when
  he has his own (a `boots.model` name, a model in `boots/`, a boots link), the two hands
  when he has gloves (a glove or hand entry, a gloves link); `roles::in_boots_folder`. A
  face listing no model of his own now gets `xml_face_neck_added` with the dummy beside the
  refkit entries (no real export reaches it). Tests: TC-REF-16 and TC-REF-17
  (`referees.rs`), the member xml naming one refkit part, the five own-boots and own-gloves
  cases, a refkit hand listed by hand, a team player's `fpc_off` (`prefox_faces.rs`), the
  planning unit test. Files: `team_compiler` `plan/mod.rs`, `plan/roles.rs`,
  `processing/prefox_face.rs`, the `ModelFolder` test constructions, `tests/cli/referees.rs`,
  `prefox_faces.rs`. Gates green (`.tmp/4_y/gates_fix14b.log`, acceptance 308 of 314);
  mutants 26/0/3 (`.tmp/4_y/mutants_fix14b.log`). Verify re-run at the close: TC-CMN-18,
  TC-REF-15, TC-REF-16 and TC-REF-17 proven, red first; the refs census as (a2) reports it.

- [x] 4.y-fix15 **S14's SWE-2 rework** (rulings S14.1 in `.tmp/4_y/duck_rulings.md`).
  (1) Under `ingame_face` the deep pass's `.mtl` check sees the textures of the combined
  `Boots/` and `Gloves/` folders: landed with 4.y-fix17 (`ReadFolder`, the test
  `under_ingame_face_a_texture_only_the_combined_boots_folder_holds_supplies_his_own_mtl`).
  (2) A `.face` link gives the player a face only when the linked `Faces/` folder gives
  one (`roles.rs` `FolderModels::of_player` and `part_source_models` call
  `with_linked_face` for any face link): today a texture-only face folder's link packs
  the player's own `face_diff.bin` into the blank face on both engines, where the plan
  gives the blank face the bundled diff and calls the file `face_file_not_used`. (3)
  Under `ingame_face` on PES 15-17 a `.common` link to a per-kit variant has no role:
  landed with 4.y-fix16 (`pre_fox_link`'s `kit_token` guard serves both arms, the test
  `under_ingame_face_a_link_to_a_per_kit_common_model_is_not_used`). Plan only (done
  with this step's opening): the
  `face_file_not_used` row names a member's `face.xml` under the marker and gives
  `fcl_hair_sim.fclo` to Fox. Plan: `player_folders.md` "`ingame_face` with shared
  links", "A link plus local models combines"; `pipeline.md` step 4 (the blank face);
  `messages.md` `face_file_not_used`, `mtl_texture_not_found`; DECISIONS 2026-10-09 (a
  `.common` link to a per-kit variant). Crates: tc
  → verify: a test per item, red first; S14 gets a second SWE-2 round on the rework diff.
  Done 2026-10-10 (Opus 5.5, one run with 4.y-fix19): item (2): `FolderModels::of_player`
  takes the export, resolves each face link (`linked_folder`) and counts it as a face of
  the player's only when the linked folder's own models give one (`of_shared(...).face`);
  `part_source_models` computes the combined folders first and asks the same of a combined
  face; `common_models` takes the export. Test: `face_folders.rs`
  `a_link_to_a_face_folder_holding_textures_alone_gives_no_face` (the tracer's boots and
  diff with a `Round.face` link to a folder holding `skin.dds` alone: PES 21 `check` and
  `compile` give `face_file_not_used` and the blank face, `link_combined` standing, the
  folder's textures being a source of his; PES 17 with the boots removed, since a direct
  `boots.fmdl` is a typed face part there, gives the dummy pair and the bundled diff).
  Contradictions ruled: the PES 17 half cannot keep the brief's boots (accepted, the test
  removes them); the test cites no acceptance ID, TC-MOD-32's scenario having no link
  (accepted: the `face_file_not_used` row already states the link case, no new scenario).
  Files: `plan/roles.rs`, `plan/mod.rs`, `validation.rs`, `tests/cli/face_folders.rs`,
  `prefox_faces.rs` (`expected_face_xml` shared). Gates green (`.tmp/4_y/gates_fix15.log`,
  acceptance 304 of 313); mutants over both steps MUTANTS (`.tmp/4_y/mutants_fix15.log`).

- [x] 4.y-7z **A small solid `.7z` export stays decompressed from its check to its tasks**
  (the maintainer's answer, 2026-10-10). `compile` keeps an archive's permit and buffers
  from its check to its tasks when it is at most an eighth of the cap and, with it kept,
  `MIN_FREE_BUDGET` (1 GiB, a `pipeline` constant) of the cap stays free of kept
  archives; any other archive is opened again as now. Plan: `libs/pipeline.md` "What a
  solid `.7z` is charged"; DECISIONS 2026-10-10. Crates: pipeline, tc
  → verify: a test that a kept archive is decompressed once per run (a counting reader or
  the budget's `peak`), one that an archive over the eighth or past the floor is opened
  twice, red first; the 4.7f timing re-run on a 7z export (about 2 s less); the largest
  single task charge over the census exports measured against the floor (the plan's 0.45
  to 0.7 GB for an 8192x8192 texture is computed from `decode_charge`, not measured).
  Done 2026-10-10 (Opus 5.5, one run and two rework rounds). `pipeline`: `MIN_FREE_BUDGET`
  (1 GiB) and `MemoryBudget::cap()`. `validation.rs`: `keeps_archive(size, kept, cap,
  largest_other)`, `CheckedSource::content` (a `.7z` kept with its permit, `None` for a
  folder, a zip, a refused archive or under `check`), `validation_pass(keep_archives)`
  (`compile` true, `check` false), the serial `.7z` loop deciding each keep with a running
  total; `compile.rs`: `CompileSource { source, revision, content }` for the coordinator,
  whose `.7z` branch reads the kept buffer or opens the archive as before; a refused
  duplicate team lets its kept content go. The sidekick's contradiction, accepted as a
  decision: the floor covers charges, not the acquires that wait (a later `.7z`'s check,
  an earlier archive in the coordinator, an archive over the cap), so the rule also
  reserves the run's largest other `.7z` (DECISIONS 2026-10-10 "a kept archive leaves
  room"; `libs/pipeline.md`; `team_compiler/pipeline.md` step 3 and "Admission"
  rewritten). Tests: `keeps_archive` cases, the two-fixture loop test, the coordinator's
  `peak < 2 * S` with the tracer `.7z` kept (red: exactly twice S), the CLI test of the
  tracer compiled kept and refused (`memory_cap_percent` 0.01) to one CPK. Lead
  measurements: the largest single charge over the census roots and the two timing
  exports is 427 MiB (FNG's 8192x8192 decode; the census's 53 VGL26 exports are old-layout
  and run no task; `.tmp/4_y/charge_probe/RESULT.md`), 0.42 of the floor; the timing
  re-run (`.tmp/4_y/timing_7z/`, release builds of `b5bd74f` and this tree, median of 3):
  compile of the FNG `.7z` 7.74 to 5.55 s and of the DBG one 6.48 to 4.57 s, the folder
  compiles and the checks unchanged within noise. Gates green (`.tmp/4_y/gates_7z_r3.log`);
  mutants 36, 26 caught, 10 unviable, 0 missed (`.tmp/4_y/mutants_7z_r2.log`; r1: 37, 24
  caught, 2 missed, the floor's pin and the loop's running total, both tested in rework 2).

- [x] 4.y-harness **The in-game harness moves into `scripts/ingame/`** (maintainer,
  2026-10-09). Lead: `game.py` (launch, screenshot, keys, close) and `burst.py` (frame
  bursts) from `.tmp/4_0/ingame/`, and one `install.py` generalizing the per-check
  install and revert scripts (`.tmp/4_19/ingame/test_ref04.py`: a CPK into the test slot,
  the installed copy kept aside and restored by hash), installing the compiler's own output
  CPK, so the scratch `apptest` tool's extract and merge are not needed; a `README.md`
  with the procedure (the slot, the key sequences to Edit mode and to a walkout, what is
  never touched: saves, `DpFileList.bin`). Paths of the installs as settings at the top,
  not buried. → verify: TC-REF-04's check re-run through it on PES 17, the slot restored
  by hash.
  Scripts landed 2026-10-09 (Opus 5.5 subagent, one round): `scripts/ingame/` `game.py`,
  `burst.py` (imports `game.cmd_shot` instead of a subprocess), `install.py` (each copy through
  a `.partial` file replaced in once its hash matches; the kept original and the last-installed
  hash in `.tmp/ingame/pes17/`), `README.md` (the Edit-mode sequence recovered from the Test 3
  transcript). The old `apptest merge -` had no base CPK, so installing a whole compiled CPK
  loses nothing. Exercised against a scratch slot only (refusals, a round trip); the verify
  run on PES 17 waits for 4.y-ingame. A check that needs a hand-built CPK builds it first.
  Done 2026-10-10: verified by 4.y-ingame's three runs on PES 17 (install, launch, capture,
  close, revert by hash), standing in for TC-REF-04's re-run. Added then: `pad-serve`,
  `pad`, `pad-stop` (one virtual Xbox 360 pad kept plugged in, `vgamepad`) and `obs-shot`
  (OBS's frame of its program scene), after the maintainer's ATF bot: neither takes the
  keyboard focus, and PES 17 reads the pad in the background once its settings use
  XInput. `keys` and `shot` stay as fallbacks.

- [x] 4.y-ingame **Two in-game checks on PES 17, by the lead** (the maintainer's answers,
  2026-10-09; through `scripts/ingame/`, after 4.y-harness). (a) A converted collar: an
  FMDL collar compiled for PES 17 (its first material `uni_collar`, the rest `uni_shirts`,
  TC-CMN-09) drawn with the kit texture as the stock collars are, in Edit mode. (b)
  `dummy_kit`: a face model converted for PES 17 whose material the conversion points at
  `model/character/uniform/common/<team>/dummy_kit.dds` (4.17a) shows the kit texture on
  the modded exe. Each result is a `manual: checked` line here and, when it contradicts the
  plan, a decision entry and a step. → verify: both checked, with frames kept under
  `.tmp/4_y/ingame/`.
  Done 2026-10-10 through the harness, driven by a virtual pad and OBS (frames in
  `.tmp/4_y/ingame/frames/`; the CPKs compiled by the release CLI at `e2eff85` as
  `4cc_90_test`, PES 17 install read; slot restored by hash after each run).
  manual: checked 2026-10-10 (a): PES 21's stock `collar_107.fmdl` as `Collars/collar_039.fmdl`
  of an `/a/` Midcup export, converted for PES 17 (`uni_collar` then `uni_shirts`, two
  meshes), drawn on the keeper WIDEFACE in Edit mode as the laced polo it is, in the GK
  kit's black (`q03_neck.png`); the baseline, PES 17's own `collar_039.model` packed as
  it is, draws a plain stand collar (`r02_neck.png`).
  manual: checked 2026-10-10 (b): the tracer's `fcl_hair.fmdl` converted for slot 01 and
  02, its body material naming `common/702/dummy_kit.dds`, draws kit 2's texture on
  both (the keeper wears the GK kit, slot 02 kit 1), and a green checker shipped at that
  path is not read (`p20_appearance.png`, `q04_slot02.png`): the plan's "active kit"
  corrected, DECISIONS 2026-10-10; no compiler change.

- [~] 4.y-ingame2 **The remaining in-game checks, by the lead** (the maintainer,
  2026-10-10: every `QUESTIONS.md` "In-game checks" entry but the face-diff
  investigation; PES 21's test slot `download/4cc_99_test.cpk` and PES 19's
  `download/4cc_69_midcup.cpk`, both the cup's placeholder, in `install.py`
  `TEST_SLOTS`, PES 19 in `game.py` `GAMES`, the triggers as pad inputs). Each result
  is a `manual: checked` line here, its QUESTIONS entry removed or narrowed, and a
  decision and a step when it contradicts the plan. Open: (a) the stock collar sets'
  look and the two checks' PES 17 half on a standard-body team whose boots do not
  cover the socks (`/out/`'s gaiters hide the lower sock); (b) the pre-Fox referee
  marker scene (PES 17, run D of `.tmp/4_19/ingame/test_ref04_runs.py`); (c) the
  192x512 mipped texture and a `.model` mesh tag (PES 17); (d) the Fox cube map of
  type 0xD (PES 21, a Fox FMDL pointed at a converted `env.ftex`); (e)
  `kit_config::validate`'s two Infos (a shirt model outside 144, 160 and 176; an
  undocumented sleeve value; PES 17 and 21); (f) a Fox subfolder texture before
  `sourceimages` (`jessie/sourceimages/#windx11/skin.ftex`, 4.y-sub (b3), `QUESTIONS.md`
  "In-game checks"; PES 21). → verify: each a `manual: checked` line
  with its frames under `.tmp/4_y/ingame/frames/`.
  manual: checked 2026-10-10, the portrait header variants on PES 21: the four files
  of `C:/Data/4cc/Tools_Mine/temp/variants/` as the portraits of `/out/` bench players
  (`.tmp/4_y/ingame/build_portraits_cpk.py`), each hovered alone in Kick Off's team
  sheet after its neighbour (the maintainer's method): A (DX10 header, BC3, sRGB id,
  one level) and B (the same with the UNORM id) crash the game on hover; C (the
  legacy `DXT5` header) and D (BC7 under DX10, full chain) draw
  (`v21_portraits_sheet.png`). So the DX10 header itself, not the colour-space id,
  crashes the Fox games' BC3 portrait reader, and the compiler's re-headering (4.y-fix6)
  is the right fix; BC7 stays under DX10, which draws. The cup's eight BC7 portraits (ESG
  XXX07-11, FGOG XXX13/21/23) are 128x128 with eight levels under DXGI 98, D's shape, so
  nothing of the question is left (its QUESTIONS entry removed).
  manual: checked 2026-10-10, collars beyond the stock set on PES 21: Test 3's CPK
  (`.tmp/4_0/apptest/out_test3/pes21/`, team `/out/`: kit 1's collar 200, a copy of the
  game's `collar_107.fmdl` under that ID; the GK kit's collar 201, no model anywhere)
  in the test slot, Edit mode's Appearance view: the outfielder draws a folded collar
  (`p21_19_collar_front.png`), the keeper a plain neckline, no crash
  (`p21_22_gk_front.png`): an ID beyond the stock set loads when its model exists.
  manual: checked 2026-10-10, the sock table on PES 21, in part: sock design A
  (`socks_design_prefox.png`, converted) on `/out/`'s kit 1: the thick white stripe at
  the front, the thin red one and the yellow disc on the outer side, both socks alike,
  the stripes straight, no seam at the visible part (`p21_legs_sheet.png`); the
  diagonal and the hoop sit under the team's gaiter boots, and the PES 17 reference
  view is still to take (open item (a)).
  manual: checked 2026-10-10, the sock table's PES 17 reference and the lower sock:
  Test 3's kit (design A, laid out for PES 15-17, so no conversion) compiled for
  `/out/` on PES 17 with the cup's own heads, whose PES 17 players wear the game's
  boots: the thick white stripe at the front, the thin red one at the back, the
  yellow disc on the outer side, the green diagonal band on the lower calf and the
  orange hoop at the ankle, as drawn (`s17_34_out_legs.png`, `s17_35_out_legs_side.png`,
  `s17_36_out_legs_back.png`); PES 21's converted upper part matches it feature for
  feature. The lower part on PES 21 still waits for a player without the gaiters: a
  `settings.toml` with `boots_id = 0` writes no table before Phase 5, and a blank
  `face.fpk` in the test slot did not replace `/out/`'s cup heads in PES 21's Edit
  mode (`w21_24_roster.png`), an observation for Phase 5's own checks. Method:
  `/a/`'s save hides its players' standard bodies (billboard models), so any face put
  on an `/a/` slot draws with no body on PES 17; the Face view draws the face package
  alone, and the whole body shows in the Appearance view zoomed out or in Strip Style
  (`scripts/ingame/README.md`).
  manual: checked 2026-10-10, the stock collar set: the template config's collar and
  the version's last stock ID (116 on PES 17, 131 on PES 21) each draw a collar on
  `/out/`'s standard bodies, a crew neck with the kit's own V at the back, 116 a
  V-neck under the long-sleeve toggle (`s17_38_collar_front.png`,
  `s17_42_c116_collar.png`, `s17_47_sleeves_long.png`, `w21_17_row5.png`); the
  QUESTIONS entry closed.
  manual: checked 2026-10-10, `kit_config::validate`'s two Infos: shirt model 150
  (outside the documented 144, 160 and 176) draws a sleeveless shirt on PES 17 and
  PES 21, no crash (`s17_40_model150.png`, `w21_17_row5.png`), a model the game has,
  so `kit_shirt_model_unknown` names a documentation gap, not a bad value;
  `long_sleeves = 3`, an undocumented raw value, leaves the short-sleeve view as it
  is and draws the long-sleeve variant without sleeves on PES 17
  (`s17_47_sleeves_long.png`), no crash. Both stay unreported by the compiler
  (DECISIONS 2026-10-10); open item (e) closed, (a) closed but for PES 21's lower
  sock.
  manual: checked 2026-10-10, the 192x512 mipped texture (c, first half): galosengen's
  `Common/scroll.dds` (192x512, DXT5, ten levels) compiles for PES 17 with no finding
  (the pre-Fox rule has been `texture_not_div4` alone since 2026-10-07; the question's
  "both engines" was stale) and, as the `face.dds` of the Fumos head on eight `/out/`
  players, draws its content (black with brick-red squares, `scroll_source.png`) on
  their heads in Edit mode's roster preview (`t17_rows_sheet2.png`, rows 16-23, against
  the fixture's red texture on rows 1-15): PES 17 reads a mipped texture whose sides are
  not powers of two, so `texture_not_pow2` stays Fox's. Export built by
  `.tmp/4_y/ingame/build_tagtex_export.py`.
  manual: checked 2026-10-10, the `.model` mesh tag (c, second half): the stock
  `konami_modD_cap.model` is PES 17's captain armband (its one mesh tagged
  `DCaptainmark`, material `modD_cap_phone` with no texture), put as `cap.model` on
  `/out/` players with a blank face: the tagged one and a copy with the tag renamed
  `DZaptainmark` both draw the white band on the left arm in Edit mode's Appearance view
  (`t17c_face_01.png`, `t17c_face_08.png`; none on the control, `t17c_face_16.png`), and
  in an exhibition match (/acccc/ against /wsg/, the tagged band on /wsg/'s slots 01-11
  with blank faces, the untagged one on 12-23 under the Fumos head) a headless
  non-captain wears the band in the replay's ground camera (`t17m_free_left.png`; the
  captain is Ricardo Milos, red-headed in `t17m_rt_22.png`). The game draws a tagged mesh
  as an untagged one, so the IR keeps dropping tags (`ir.md`, DECISIONS 2026-10-10); the
  QUESTIONS entry removed. (c) closed.
  manual: checked 2026-10-10, a `.model` collar converted for PES 21 (4.y-fix12's open
  question): PES 17's stock `collar_001.model` and `collar_050.model` (the probe copies in
  `.tmp/4_y/uniform_mtl/`) each as `Collars/collar_012.model` of an `/out/` export with
  Test 3's kit 1 (`.tmp/4_y/ingame/build_collar21_export.py [050]`, the release build of
  `104d8c8`; 200 is `collar_id_invalid`), the CPK's `collar_012.fmdl` in the test slot: row
  19's standard body draws the same plain neckline for both (`c21_14_row19_neck.png`,
  `c21_24_row19_neck.png`), the one a collar ID with no model draws (`p21_22_gk_front.png`),
  where the stock `collar_107.fmdl` copy drew its folded collar (`p21_19_collar_front.png`).
  The converted FMDL (the converter's `fox3ddf_blin`, every sampler dropped) is not drawn:
  step 4.y-fix18 gives it the stock collars' Fox material set. The PES 21 Edit path drifts
  when the title needs one `a` instead of two (Kick Off's submenu opens: `b` backs out).
  manual: checked 2026-10-10, the same `collar_050.model` under ID 12 with 4.y-fix18's
  release build (the stock collars' Fox material set): row 19's standard body draws a
  folded collar in the kit's colour at the neck (`f18_05_row19_neck.png`, against
  `c21_24_row19_neck.png`'s plain neckline), so the material set was the whole cause;
  the missing vertex colour stream and the four unweighted bones the converted FMDL lacks
  (4.y-fix18's comparison) change nothing visible. Compiled with the CLI from the test
  folder (`team-compiler compile --no-deploy --export`; the version is the settings').
  (d) answered by the maintainer 2026-10-10, no run needed: a Fox referee export's `env.dds`
  is a pre-Fox leftover, and no Fox material samples a cube map
  (`resources/Materials.wikitext`), so the 0xD form is drawn by nothing and stays Red's
  harmless output (`pipeline.md` step 3); the QUESTIONS entry removed. The earlier note:
  manual: not run 2026-10-10, the Fox cube map of type 0xD (d): no material of the refs
  export (robocopclassic's `apc.mtl`, `face.mtl`) and no bundled `.mtl` names `env.dds`,
  and the 4cc Fox CPKs hold no cube map (4.y-fix3's recon), so a Fox material pointed at a
  converted `env.ftex` exists in no export: the check needs a hand-made FMDL, which no
  reachability rule puts ahead of the queue. The QUESTIONS entry stands as written.
  manual: checked 2026-10-10, the pre-Fox referee marker (b), no answer yet: run D (the
  checker as `parts/referee/incom_bsm.dds`) and run G (the template tree's
  `referee_prop.model` and `.mtl` with it, what the compiler writes) in PES 17's test
  slot, an exhibition match of `/4cccc/` against `/wsg/` driven by the pad: no magenta
  and yellow square anywhere in the walkout, the kickoff's wide camera or the replay's
  top-down and free cameras at 0:00 (`r17_walk_*`, `r17g_kick_*`, `r17g_top_03.png`),
  and no referee figure at all in any of them (a player leads the walkout; the only
  figures on the pitch wear the two kits). The cup's own `4cc_35_referees.cpk` holds the
  same prop pair and its clover `incom_bsm.dds` (5.6 MB), 35 referee face CPKs and the
  referee kit textures, and Sider's LiveCPK root (`C:/Data/4cc/0Livecpk`) holds no referee
  file, so the install's referee is drawn by nothing the lead can see: whether the
  referee is hidden on purpose on this install, and which scene draws the prop, stays
  the maintainer's (QUESTIONS, narrowed). The replay's free camera (mode 3 of the A
  button) is tethered to the target player and the pad holds one input at a time, so
  no close view of an official was reachable; the pad's `hold:left` rewinds there.
  manual: checked 2026-10-10, a Fox subfolder texture before `sourceimages` (f): /@/'s
  `hair_high.fmdl` (a whole head naming `face_bsm`) on /out/'s slots 01-03 in PES 21's
  test slot (`.tmp/4_y/ingame_f/build_subtex_export.py`, compiled by the release build of
  `565c802`): slot 01, the head and its own texture in the root, draws as /@/'s player;
  slot 02, the head and a magenta `face_bsm.png` in `jessie/`, (b1)'s layout as compiled
  (`.../02 - Old/sourceimages/jessie/#windx11/`), draws magenta; slot 03, the same in
  green with the CPK patched to the chosen layout (`.../03 - New/jessie/sourceimages/
  #windx11/`, the FMDL directory moved alike, `patch_new_layout.py`), draws green
  (`s21_rows.png`, Edit Player's roster preview). The game reads a texture wherever the
  FMDL's directory says, `#windx11/` inserted before the name, so slice (b3)'s layout move
  is safe; (f) closed, its QUESTIONS entry removed.
  manual: checked 2026-10-10 by the maintainer, the pre-Fox referee marker (b): PES 17 with
  the cup's current `4cc_35_referees.cpk` (the template's prop pair and clover, what the
  compiler writes, step 4.19f) draws the clover on the ground under each of the three
  officials in every scene (a walkout in the black Void stadium, the maintainer's
  screenshot). That CPK's clover is a plain 5.6 MB DDS, the form the compiler writes there
  (`blue_port.md`), so the plain-DDS half is answered too. Why the lead's runs D and G (a
  plain checker through the test slot) showed no marker is not known: the marker does not
  scale with a small referee (the maintainer), so a missed official does not explain it.
  (b) closed, its QUESTIONS entry removed.

- [x] 4.y-fix16 **S9's second Astra round: the deep pass checks what `compile` reads,
  five more places** (rulings S9.A2 in `.tmp/4_y/duck_rulings.md`, all five accepted,
  each traced in the code). (1) `folder_findings` reads a folder's roles with the
  `ingame_face` marker (`FolderModels::of_player_files(.., marker, ..)`), so a marked
  PES 15-17 folder's left-out higher kit variants are unread; and planning's marker
  branch (`roles.rs` ~1138) keeps `pre_fox_link`'s kit-token rule (a `.common` link
  to a per-kit variant has no role, `file_not_used`), closing the `mtl_for` panic at
  `plan/mod.rs` ~1941. (2) A beaten Common representation is dropped with its dropped
  winner (one Info naming the winner), so planning never selects a file the pass did
  not check; both engines; decision entry. (3) In an xml-controlled pre-Fox source the
  models and `.mtl` files the xml does not reference are unread (a referenced set's
  variants count as referenced). (4) On Fox a Common `.mtl` is unread unless a selected
  `.model`'s conversion or a Common `.model` link's search reads it; `messages.md` ~278
  reworded to match ~330. (5) A refs export's kits, logo, portraits and collars are not
  checked, as its root colors are not. From 4.y-fix11's review (2026-10-10): (6) the
  deep pass reads a player's models by his folder's roles alone, so a variant left out
  only across sources (a combined `Boots/` folder's `boots_kit2.fmdl` beside the
  player's `boots_kit1.fmdl`, which his boots package leaves out) is still read and
  checked, and a broken one drops the folder: the pass takes the roles over the
  sources, as `ModelFolder::roles` does. (7) `roles::admitted` (`validation.rs` ~795)
  goes through `role_position`, so a file below a shared folder's subfolder is
  `file_type_disallowed` alone, not `file_not_used` again (the fix11 test in
  `models.rs` pins the double report today: update it). (8) `prefox_face.rs` ~421 and
  `prefox_shared.rs` ~445 search a combined source's `.model` with the player's
  `common_files`, as `processing/model.rs` did before fix11 item (2): the same rule,
  a test per site on PES 17 (a combined `Boots/` holding `materials.mtl` and a
  name-matched `boots.mtl.common` link), reported if no failure reproduces. (9) fix11's
  mutation survivor: `ModelFolder::roles`'s `part_sources(index + 1)` (`plan/mod.rs`
  ~319) with `*` for `+` gives a pre-Fox face's first combined folder the player's own
  files as its source, so the folder's own set is not left out; test: a PES 17 referee
  linking `Boots/Studs` holding `boots_kit1.model` and `boots_kit2.model`, whose
  `k99NN` boots hold kit 1's model alone. Plan: `pipeline.md` "Deep validation" (~105),
  `messages.md` `file_not_used`, `xml_model_unlisted`, ~278; DECISIONS 2026-10-09 "One
  rule". Crates: tc
  → verify: a test per item, red first (a marked folder with a broken higher variant
  compiles; a kit-token link under the marker is `file_not_used`; a dropped winner's
  beaten file is not converted, both engines; a broken unlisted model beside a valid
  xml; a malformed Common `.mtl` beside an FMDL player with a `.mtl.common` link on
  PES 21; a refs export with a broken copied kit config exits 0); S9 gets a third
  Astra round after the rework, with its SWE-2 loop.
  Done 2026-10-10 (sidekick, one run; brief `.tmp/4_y/brief_fix16.md`, report
  `.tmp/4_y/sk_fix16_report.md`): `roles::part_source_models` computes a folder's and its
  combined folders' models for planning and the deep pass alike (`FolderModels::of` gone);
  `pre_fox_link` holds the kit-token rule for both arms; a beaten `Common/` model is
  dropped with its winner (`common_model_beaten_dropped`, Info, `DropFile`; DECISIONS
  2026-10-10); `deep::xml_unnamed` leaves what an xml does not name unread (an FMDL there
  always, since the face converts none in an xml-controlled source; a `kitN` name names its
  set); on Fox `searched_common_mtls` is the set of Common `.mtl` files any player's search
  reads, the pairing rule in one `pairs_with_mtl`; a refs export's kits, logo, portraits
  (the folder's too) and collars skipped with its root colors; `admitted` through
  `role_position`; the two pre-Fox search sites pass no Common files for a combined source
  (both panics reproduced). The rulings' shapes held but for the red runs (a `boots.mtl`
  name-matched the link's search; a `.fmdl.common` link was already dropped, the hole was a
  `.model.common` one; the mirror reports `model_material_undefined`, not a cascade) and
  (8) b's output name (`boots.mtl`). The shared folders' own checks stay over their own
  files: one check serves every linker, and a plain link or the shared output reads the
  folder whole, so a file left out only in one linker's context is still read (accepted
  residual). Follow-up to the next rework brief: `held_stems` takes its shared folders
  from `FaceUse`, `Unused` under the marker, so a marked player's combined boots or gloves
  textures do not count for his own `.mtl` paths (a possible false `mtl_texture_not_found`,
  folder kept): build the list from `combined_folders`. Files: `deep/mod.rs`,
  `messages.rs`, `plan/mod.rs`, `plan/roles.rs`, `processing/prefox_face.rs`,
  `processing/prefox_shared.rs`, `user_face_xml.rs`, `validation.rs`, five CLI test files;
  `pipeline.md`, `messages.md`, DECISIONS (lead). Gates green (`.tmp/4_y/gates_fix16_lead.log`,
  acceptance 301 of 312); mutation diff (`.tmp/4_y/mutants_fix16.log`): 40 mutants, 33
  caught, 7 unviable, 1 missed (`drop_beaten_common_models`' `is_direct_model`, `&&` to
  `||`: a dropped Common texture of the stem would count as the winner), closed by a lead
  CLI test in `common_links.rs` (red with the perturbation by hand, reverted); the remote
  half peaked at 8.60 GiB of the 9G cap. Found by that test's first shape: a `Common/`
  texture whose bytes carry no header the deep pass knows (four junk bytes) gets no finding
  and fails the Common textures task (`folder_pack_failed` at `Common`, every Common
  texture lost): issue below.

- [x] 4.y-sub **A subfolder of a player folder is a player folder of its own** (the
  maintainer's rule, DECISIONS 2026-10-10, two entries; plan `player_folders.md`
  "Subfolders", `object_model.md` "File-type allowlist", `messages.md`
  `player_layout_proto`, `texture_stem_conflict` and "User-supplied `face.xml`",
  `pipeline.md` step 6, `model_format.md`, `blue_port.md`, `export_upgrader.md` items 9
  and 12). Briefed in slices, each its own commit:
  - (a) roles and the allowlist: the reserved subfolders removed from `roles.rs` (`position`,
    `model_role`'s forced categories, `face_file`), a subfolder's files typed by their names at
    any depth, the root's markers, settings and links alone counting, a shared folder's
    subfolder files `file_type_disallowed`, the refs `player_layout_proto` refusal (TC-MOD-67,
    TC-MOD-68, TC-REF-18). Done.
  - (b) the pre-Fox face: paths kept in the face CPK and the texture home, `face.xml` entries by
    path, `./sub/name` a checked local reference, the `.mtl` search's folders, the nearest-first
    texture lookup and the per-folder `texture_stem_conflict` namespace (TC-MOD-66). Landed as
    (b1) and (b2).
  - (b3) links mirror `Common/`'s tree, a model names a subfolder's texture by its path, shared
    folders take subfolders (the maintainer, 2026-10-10, DECISIONS "links mirror `Common/`'s
    tree; a model names a subfolder's texture by its path; shared folders take subfolders and
    links", superseding the dotted-prefix rule of the same day): validation admits every link
    below a subfolder and resolves a `.common` link against `Common/<its path>` (`links.rs`, the
    allowlist's rows), `role_position` reads links at any depth, `shared_link_duplicate` counts
    the tree (one folder twice counting once), `common_file`/`common_link_name` take the link's
    path, `mtl_search` and `linked_common_materials` load a nested material link's target
    (4.y-fix21's former item (1); every panic site S18.A1-1 and S18.1-C4 named:
    `processing/model.rs`, `prefox_face.rs`, `prefox_shared.rs` `material_of`), the Common tasks
    pack a linked subfolder file at its path, before `sourceimages` on Fox
    (`is_read_common_file`), a player's subfolder texture moving there too from (b1)'s
    `sourceimages/<sub>/#windx11/` (`TextureHome::texture`, the FMDL directory; DECISIONS "on
    Fox a subfolder's texture sits before `sourceimages`"), and the pre-Fox Common output lists
    it at its path (TC-CMN-20, TC-MOD-67, TC-MOD-70); a texture named with a path below the
    model's folder resolves at that path, an FMDL directory starting with `./` read as one
    (`./textures/`, TC-MOD-71), and a shared folder's subfolders follow the player folder's
    rules (TC-MOD-68 reworded); TC-CMN-13 as it is. Sliced:
    - (b3a) the texture named by its path and the Fox layout move. Done.
    - (b3b) nested `.common` links and the Common tasks, sliced:
      - (b3b1) model and material links. Done.
      - (b3b2) texture links. Done.
      - (b3b3) a player's own texture or texture link winning over a combined folder's with
        `shared_texture_overridden` (DECISIONS "a player's own texture or link wins over a
        combined folder's, as an Info"). Done.
      - (b3b4) a `Common/` subfolder `.mtl` or model resolving its textures in its own `Common/`
        directory, sliced:
        - (b3b4a) PES 15-17. Done.
        - (b3b4b) Fox, where a linked model's or `.mtl`'s directory's textures travel with it.
          Done.
        - (b3b4c) S18.A3's fixes (`.tmp/4_y/duck_rulings.md` S18.A3-1 to -3): a linked Common
          file brings the textures of each directory above its own too; a player `.mtl` over a
          Common model falls back to the Common model's own directory; on Fox a Common texture
          path is pointed at Fox's Common output, installed-supplied included. Done.
        - (b3b4d) S19.1-A1's code half: a path into a subdirectory of the team's Common output
          (`common/000/kit1/sourceimages/`, written by a Fox model, not only respelled from a
          pre-Fox path) is checked like a direct one, and the installed-CPK lookup looks at the
          path's subdirectory, a `kitN/` one standing for its set (S19.2-A1) (it looks for
          the stem at the Common root today, both engines;
          `pipeline.md` "A texture a model names must exist", `messages.md`'s paragraph after
          "Texture existence is checked **deep**").
          With it S18.3-1 (the deep pass's Common places lack the folder's texture-link stand-ins
          the task's have), -4 (`point_texture`'s `_ =>` on `Reference`), -5 (a local shadowing
          `roles::common_texture_below`); and the pre-Fox `Common/` half of the check, not only the
          installed one (S18.3-2). Done; a `kitN/` subdirectory went to (b4).
        - (b3b4e) PES 15-17: a texture `.common` link whose target is not in the export
          satisfied by an installed CPK, as on Fox (the face task then points the link by its
          own stem; `messages.md`, the same paragraph's end). Deferred until the cup next
          plays a pre-Fox version (DECISIONS 2026-10-11): not a gap of this phase.
    - (b3c) shared folders' trees, sliced:
      - (b3c1) a shared folder's subfolders (TC-MOD-68, TC-MOD-73, TC-MOD-74). Done.
      - (b3c2) its `.common` links at any depth, as a player folder's (plan rewritten
        2026-10-11: `pipeline.md` step 2's face-file paragraph), sliced:
        - (b3c2a) `aesthetics_export` admits, resolves and checks a shared folder's links
          (`common_link_missing` on the shared folder, the drop cascading to its linkers);
          on Fox its texture links count, in its own output and combined into a player's
          (TC-MOD-75, TC-MOD-76); a link with no role there is `file_not_used`. Done.
        - (b3c2b) Fox: its model and material-file links, in its own output and combined
          into a player's (TC-MOD-79 to TC-MOD-82): Common models per source, the shared
          output's own Common lists, a shared folder holding only a model link has a model,
          the deep pass's `.mtl` searches; a Common file linked by both a player and his
          combined shared folder is one part, the player's own link winning (step 6). The
          hidden-model drop is PES 15-17's (`pipeline.md` step 2), so it waits with that half.
          Done.
        - PES 15-17: deferred until the cup next plays a pre-Fox version; a shared folder's
          link is `file_not_used` there (TC-MOD-77).
  - (b4) kit-token sets (`model_format.md` "Kit-dependent assets"), sliced:
    - (b4a) the stem form's `kitF` fallback: a set's fallback is its `kitF` member, else its
      lowest variant; on PES 18-21 it is written once under the reference's spelling,
      whatever numbers the export defines, instead of a copy per gap; on PES 15-17 it is
      copied into each gap; a `kitF` file is never written under its own name and a path
      naming one is not found by any check (TC-CMN-04's new THEN, TC-CMN-26 to TC-CMN-28).
      Done.
    - (b4b) kit-token folders (`kit1` to `kit9`, `kitF`) form variant sets (`kitN/<name>`)
      in any folder, sliced:
      - (b4b1) a `.common` texture link's target looked up set-aware, stem and folder form
        (the token in any directory segment): a path naming `kitF` finds nothing, else the
        literal file, else the set's members, its lowest variant (else `kitF`) the one a
        single lookup gets and every member linked (TC-MOD-72; Astra B4A.A1-2). Done.
      - (b4b2a) a folder-form set completed against the export's kit numbers as a stem
        set is, in any folder, its paths written verbatim (TC-CMN-21: /hdg/'s per-kit Common
        folders): a kit-token folder's `Common/` textures join the textures task of the
        directory above it, so `Common/kit1/` and `Common/kit2/` complete as one set; test
        mode keeps a texture's subfolders. Done.
      - (b4b2b1) a texture path into a `kitN/` directory, or naming a `kitN` stem, supplied
        when a file of its set is, `kitF` included, and a path naming `kitF` found by none, on
        both engines: every texture key set files a member under its set's `kitN` key too
        (`kit_lookup_keys`), the `kit_reference_directory` guard retired; an installed CPK's
        `kit1/` copy supplies `kitN/` (TC-CMN-29, TC-CMN-30). Done.
      - (b4b2b2) a `.common` link to a kit set dropped only when every member it reaches is,
        a dropped member an absent one the completion fills (`aesthetics_export/object_model.md`
        "Dropped link targets"; Astra B4B1.A2-2; TC-CMN-31). Done; one Astra round covered
        (b4b2b1) and (b4b2b2) together (B4B2B.A1).
      - A kit set split between a folder's own files and its `.common` links (a local
        `kitF/armor.dds` beside `kit1/armor.dds.common`) is served from the folder's own
        members alone, the linked variant unused: no export is known to split a set across
        the two (Astra B4B2B.A1-1).
      - A `.common` link to a per-kit model or material-file set (`boots_kitN.fmdl.common`)
        is a literal lookup, `common_link_missing`, until the per-kit model pipeline serves
        Common models (Fox per-kit packages, the fallback model, same-member format
        preference; Astra B4B1.A1-1 to A1-3): no export is known to use one.
      - The Export upgrader's half (a legacy `common/XXX/u0XXXp0/sourceimages/` path becomes
        `./kitN/` and a `kitN/<name>.common` link) is its phase's, `export_upgrader.md`.
    - PES 15-17, deferred until the cup next plays a pre-Fox version (DECISIONS 2026-10-11):
      a shared gloves or boots output, or `ingame_face`, using a `kitF` model packs it under
      its own lower-cased name and names it in its xml (Astra B4A.A1-4); a numbered model
      paired with a lone `pants_kitF.mtl` (`mtl_for`'s "any `.mtl`" kind) names a file the
      face strips (A2-1); a `kitF` fallback whose `.mtl` is a Common link gets no `.mtl` copy
      at the listed entry's local `material` (A2-2).
  - (c) Fox: a subfolder's models merged into the packages, textures under the texture home at
    their path (TC-MOD-65). Done by (b1), (b3a) and (b3c1) (TC-MOD-65, TC-MOD-68, TC-MOD-73):
    a file's role comes from its name at any depth (`roles::role_position`), so the Fox Models
    task merges a subfolder's model into its package.
  - (d) the deep pass over the tree (what `compile` reads, DECISIONS 2026-10-09 "One rule").
    Done across the slices: the deep pass visits a folder's whole tree and skips what
    `player_file` gives no role, the function planning uses.
  The Export upgrader's carry-over is Phase 6's. Moot after this: S14.1-4 and the subfolder half
  of S7.1-3 (a shared folder keeps the "directly in" rule, 4.y-fix11 item 3). Crates: ae, tc
  → verify: TC-MOD-65 to TC-MOD-68 and TC-REF-18 proven, red first; the census export
  `HAHA Aesthetic Export VTL9 i3` (`C:/Data/4cc/Lab/`, 23 players, 688 nested files)
  compiles for PES 17 with every player kept and its face CPKs holding the tree.
  (a) done 2026-10-10 (Opus 5.5, one run and one rework round; brief
  `.tmp/4_y/brief_sub_a.md`, report `.tmp/4_y/sk_sub_a_report.md`). `aesthetics_export`
  `validate/folders.rs`: `Position { Direct, Below }` (`Reserved` gone), the allowlist
  admitting model content at any depth below a player folder and the markers, settings
  and links directly in it alone (a subfolder's `file_type_disallowed`); `PROTO_SUBFOLDERS`
  and `player_layout_proto` (Error, drops the folder, context the subfolder as spelled, one
  finding per folder) for a refs export's `face`, `boots`, `gloves` or `common` subfolder;
  links resolve direct files alone; `ISSUE_CODES` row. `team_compiler` `plan/roles.rs`:
  `role_position(folder, file, shared)` (a shared folder's file or a `.common` link counts
  directly in the folder, any other file anywhere below it: the sidekick's own ruling,
  accepted, since a kept subfolder link under strict-off would reach a role nobody resolved
  and panic in `common_models`), `model_role(stem, ingame_face)` by name alone,
  `is_user_face_xml`, `named_as_face`; `Position`, `in_folder_or_face`, `in_boots_folder`,
  `pre_fox_model_type` and `admitted` gone; the role messages name a subfolder's file with
  its subfolder (`gloves/keeper.fmdl`); `prefox_face.rs` lost `boots_model` (redundant with
  the listed boots-suffix check, the sidekick's contradiction); `messages.rs` row; help
  `01_command_line.md` rewritten to the rule, with a `player_layout_proto` sentence. Interim
  until (b): a `face.xml`'s `./name` reference finds a file of that name at any depth, two
  `face.xml` in one folder both take the role (the first wins), and `texture_stem_conflict`
  runs over direct files alone. Tests: TC-MOD-67, TC-MOD-68, TC-REF-18 (`models.rs`,
  `referees.rs`), TC-MOD-14 and TC-MOD-53 rewritten to the rule, TC-STR-13 rewritten to its
  first case (a subfolder's `hair.dds` is TC-MOD-66's), TC-MOD-63 withdrawn (its premise, a
  `common/` file no task reads, is gone), the `role_position` unit test red by hand for
  another folder's file. Files: `aesthetics_export` `parse/draft.rs`, `validate/{folders,
  issues, links, tests}.rs`; `team_compiler` `deep/{documents, mod}.rs`, `messages.rs`,
  `mtl_search.rs`, `plan/{mod, roles}.rs`, `processing/{model, prefox_face}.rs`,
  `user_face_xml.rs`, `validation.rs`, `help/01_command_line.md`, `tests/cli/{common_links,
  compile, models, prefox_faces, referees}.rs`; README (lead). Gates green
  (`.tmp/4_y/gates_sub_a_r2.log`, acceptance 311 of 313, TC-MOD-65 and TC-MOD-66 waiting on
  (c) and (b)); mutants 124 mutants, 103 caught, 21 unviable, 0 missed (`.tmp/4_y/mutants_sub_a_r2.log`).
  (b1) done 2026-10-10 (Opus 5.5, one run and two rework rounds; brief
  `.tmp/4_y/brief_sub_b1.md`, report
  `.tmp/4_y/sk_sub_b1_report.md`): a subfolder's texture is emitted at its path below the
  texture home on both engines (`PlayerFile::Texture { below, format }`, `below` the path
  below the source folder without extension; `TextureHome::texture(engine, team_id, below)`,
  on Fox the platform folder in the texture's own directory, decision "a Fox subfolder
  texture sits in its own platform folder", the sidekick's contradiction accepted on Red's
  Common layout, in-game item (f)); the new `texture_lookup.rs` (`TexturePlace`,
  `TextureFolders::{insert, nearest_first, places}`, `split`, `variant`) resolves a
  texture name in the file's own folder, then each parent up to the root, then the
  combined shared folders, then the links, never down; the face task (`FacePlaces::of` per
  `.mtl`), the shared writer (`places_for`, an `ingame_face` player's parts sit in his
  subfolders: the sidekick's contradiction, accepted) and the Fox Models task
  (`point_texture` with the texture's subdirectory) point with it, and the deep pass looks
  a `.mtl` up with it (`HeldTextures::of`, `held_stems` gone); the textures task keys
  copies by path (`resolve_path`), so a root `skin.dds` and `jessie/skin.dds` are two
  entries, and only root textures form kit variant sets (a subfolder named `old_kit2_x/`
  would read as a variant); a kit token is matched as spelled everywhere now (Fox and the
  deep pass folded the name before, pre-Fox and the set completion did not: `find_token`
  matches `kit` lower-case). `aesthetics_export`: each folder of a player's tree is its own
  `texture_stem_conflict` namespace (`texture_claim`), the four singletons are
  `file_type_disallowed` below a subfolder (`pub fn is_player_singleton`, consumer
  `roles.rs` `role_position`, which gives a nested singleton no role under a lenient
  check: the sidekick's contradiction, accepted, S18.A1-2). TC-MOD-65 reworded by the lead
  (its texture sat below the model's folder, which nearest-first never reaches; now
  `jessie/shorts.dds` beside a root `shorts.dds` of other bytes) and proven by the Fox CLI
  test; nine unit tests that put a singleton in `face/` rewritten to the rule. Tests red
  first (nine runs pasted). Files: `aesthetics_export` `conventions/{mod,
  player_folder}.rs`, `lib.rs`, `validate/{folders, tests}.rs`; `team_compiler`
  `texture_lookup.rs` (new), `lib.rs`, `paths.rs`, `plan/{mod, roles}.rs`, `deep/{documents,
  materials, mod}.rs`, `processing/{model, prefox_face, prefox_shared, texture}.rs`,
  `user_face_xml.rs`, `validation.rs`, `tests/cli/{models, prefox_faces}.rs`; DECISIONS,
  player_folders.md, pipeline.md, QUESTIONS, README (lead). Gates green
  (`.tmp/4_y/gates_sub_b1.log`, acceptance 312 of 313, TC-MOD-66 waiting on (b2));
  mutants r1 99 mutants, 2 missed (the own/combined flag of the texture places at the Fox
  and pre-Fox sites, `.tmp/4_y/mutants_sub_b1.log`; the first rework's two CLI tests catch
  them), r2 99 mutants, 1 missed (`texture_claim`'s `||`: a texture link below a subfolder
  counted in its namespace, the second rework's unit test; `.tmp/4_y/mutants_sub_b1_r2.log`),
  r3 99 mutants, 86 caught, 13 unviable, 0 missed (`.tmp/4_y/mutants_sub_b1_r3.log`).
  (b2) done 2026-10-10 (Opus 5.5, one run and one rework item; brief
  `.tmp/4_y/brief_sub_b2.md`, report `.tmp/4_y/sk_sub_b2_report.md`): `Reference::Local`
  holds a path relative to the referencing file's folder (`./jessie/body/oral_x_*.model`,
  only the file name's `*` read as `win32`; an empty, `.` or `..` segment stays
  `Unchecked`), resolved among the folder's own files by that path (`FaceFiles::own_local`)
  and a linked shared face's direct files (`linked_local`, `linked_face` now the
  `SharedModelFolder`: S18.1-C3 closed), a `kitN` reference naming the set in its own
  directory (`variant_of`), `names_file` by the relative path; the generated face packs
  and lists each own model at its path (`FaceModel::directory`), its hands and converted
  material set beside it, every `.mtl` at its path, and names the `.mtl` where the search
  found it; a member's xml packs what it names at the reference's path. The sidekick's
  contradictions, accepted: the `role_position` filter stays in `resolve` (a shared face's
  subfolder file takes no role), and a pre-Fox per-kit model set is one directory's
  (DECISIONS "a pre-Fox per-kit model set is one directory's"); `supply`'s `./sub/x.dds`
  stays `Supplied` until (b3a). Rework: the generated face's `kit_variant_missing` names
  a subfolder set with its directory, as the xml route does. TC-MOD-66 proven. Files:
  `team_compiler` `user_face_xml.rs`, `processing/prefox_face.rs`, `deep/{materials,
  mod}.rs`, `tests/cli/{prefox_faces, prefox_user_xml, prefox_kit_variants,
  prefox_hand_split}.rs`; DECISIONS, model_format.md (lead). Gates green
  (`.tmp/4_y/gates_sub_b2_lead.log`, acceptance 313 of 318, the five open ones (b3)'s and
  (b4)'s); mutants r1 72 mutants, 55 caught, 17 unviable, 0 missed
  (`.tmp/4_y/mutants_sub_b2.log`), rework 72 mutants, 55 caught, 17 unviable, 0 missed (`.tmp/4_y/mutants_sub_b2_r1.log`).
  (b3a) done 2026-10-10 (Opus 5.5, one run and one rework round; brief
  `.tmp/4_y/brief_sub_b3a.md`, report `.tmp/4_y/sk_sub_b3a_report.md`): `paths::TextureDirectory`
  (prefix, subdirectory, suffix) decides a texture home's directory in one place, a Fox
  player's subfolder texture now at `.../<folder>/<sub>/sourceimages/#windx11/`, the FMDL
  naming `.../<folder>/<sub>/sourceimages/` ((b1)'s layout tests updated); an FMDL directory
  or `.mtl` path starting with `./` and carrying a directory (`texture_lookup::path_below`)
  resolves at that place alone (`TextureFolders::at`), from the model or, for a converted
  `.model`, its `.mtl` (`Part::references_from`), nothing there `fmdl_texture_not_found` /
  `mtl_texture_not_found`; the pre-Fox face's, boots' and gloves' `.mtl`s point the same way
  (`prefox_face::FolderPlaces`, `MaterialPlaces`), and the deep pass's `supply` decides it
  (`HeldTextures::at`; (b2)'s `Local`-with-`/` arm gone). The sidekick's contradiction on
  `prefox_shared` accepted (a shared folder's or an `ingame_face` player's `.mtl` follows the
  rule, as the deep pass does); on `Common/` `.mtl`s half accepted: a Fox Common part's
  `./sub/` path is `fmdl_texture_not_found` (Fox reads no Common subfolder file), while a
  pre-Fox Common `.mtl`'s stays not looked for, since the Common output holds its
  subfolder at its path; (b3b4) looks it up (and `prefox_common` still points such a path
  by name). TC-MOD-71 proven. Files: `team_compiler` `paths.rs`, `texture_lookup.rs`,
  `processing/{model, prefox_face, prefox_shared, prefox_common}.rs`, `deep/{materials,
  mod}.rs`, `tests/cli/{models, prefox_faces, textures}.rs`. Gates green
  (`.tmp/4_y/gates_sub_b3a_lead.log`, acceptance 314 of 318, the four open ones (b3b)'s and
  (b4)'s); mutants r1 38 mutants, 31 caught, 7 unviable, 0 missed
  (`.tmp/4_y/mutants_sub_b3a.log`), rework 39 mutants, 33 caught, 6 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3a_r1.log`).
  (b3b1) done 2026-10-10 (Opus 5.5, one run; brief `.tmp/4_y/brief_sub_b3b1.md`, report
  `.tmp/4_y/sk_sub_b3b1_report.md`): `aesthetics_export::common_link_target` (the path below
  `Common/` a link stands for); `counts_as_link` the one rule (a shared link at any depth, a
  `.common` link directly in the folder or below a subfolder unless it names a texture, which
  stays `file_type_disallowed` until (b3b2)); `player_links` resolving at the link's path,
  `common_link_missing` naming `Common/<path>`; `shared_link_duplicate` counting distinct
  folders over the tree, a folder linked twice one entry of the kept player's `links` and
  combined once; `ingame_face` and the cascade reading links at any depth. `team_compiler`:
  `common_file` by folded path below `Common/` (`common_skeleton`, `selected_common_model`
  beside it), `is_common_file`/`below_common`, `linked_common_files` (what Fox reads of a
  Common subfolder: a link's target and what travels with a linked model) feeding
  `is_read_common_file`, the deep pass, `file_not_used` and planning, the Fox Common tasks on
  `Common/` itself alone; `role_position`, `mtl_search` (the linked model's own `Common/`
  directory), `user_face_xml::candidate` and every `.expect` site switched; the pre-Fox face
  lists a nested link's model and `.mtl` at `common/<team>/<dir>/`. The sidekick's
  contradiction accepted: two tests the brief kept put a link in a player's `common/`
  subfolder, which now names `Common/common/...`; rewritten to their intent. Left for (b3b4):
  under `ingame_face` on pre-Fox, a nested Common `.mtl` copied into his boots or gloves is
  still named by its file name, its textures pointed at `Common/`'s direct ones. TC-CMN-20 and
  TC-MOD-70 proven. Files: `aesthetics_export` `conventions/{file_types,mod}.rs`, `lib.rs`,
  `validate/{folders,links,mod,tests}.rs`; `team_compiler` `plan/{roles,mod}.rs`,
  `mtl_search.rs`, `user_face_xml.rs`, `validation.rs`, `deep/{mod,model,pairings}.rs`,
  `processing/{model,prefox_face,prefox_shared}.rs`, `tests/cli/{common_links,models,
  prefox_faces}.rs`; model_format.md, pipeline.md (lead). Gates green
  (`.tmp/4_y/gates_sub_b3b1_lead.log`, acceptance 316 of 318, the two open ones (b4)'s);
  mutants r1 145 mutants, 124 caught, 20 unviable, 1 missed (`.tmp/4_y/mutants_sub_b3b1.log`: the lenient-kept texture link in `linked_common_files`' guard, a test added), rework 145 mutants, 125 caught, 20 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b1_r1.log`).
  (b3b2) done 2026-10-10 (SWE-2 High through `devin_sidekick.py`, the trial's first slice;
  brief `.tmp/4_y/brief_sub_b3b2.md`, reworks `_r1`, `_r2`, reports beside them): a
  `.common` texture link counts at any depth (`counts_as_link`), claims its stem in its own
  folder's namespace (`texture_claim`) and takes a role there (`role_position`);
  `PlayerFile::CommonTexture` holds the linked texture's path below `Common/`;
  `texture_lookup::TextureFolders` holds a folder's links beside its textures
  (`PlaceHome`), nearest first each folder's textures then its links, so a root link now
  precedes a combined folder's texture of its stem; `paths::common_home` and
  `common_texture` put a Common subfolder texture before `sourceimages` on Fox; the Fox
  Common tasks pack a subfolder's linked textures alone, an unlinked one `file_not_used`;
  Fox's installed Common set reads nested paths, so an earlier midcup CPK satisfies a
  nested link; `roles::common_texture_below` spells a link's path as `Common/` does. The
  sidekick's contradiction accepted (a root link's stem stays among the Fox Common
  stems, or its path would be `fmdl_texture_not_found`). Rework 1: the mutation run's six
  test gaps and a dead filter; rework 2: S18's second Astra round (S18.A2-1, a converted
  `.model`'s bare texture names resolve from its `.mtl`'s folder on Fox, as the deep pass
  and pre-Fox do; S18.A2-2, TC-CMN-13's test checks the whole CPK). TC-CMN-22 proven.
  Files: `aesthetics_export` `validate/{folders,links,tests}.rs`; `team_compiler`
  `bins/installed.rs`, `deep/materials.rs`, `paths.rs`, `plan/{mod,roles}.rs`,
  `processing/{model,prefox_face,prefox_shared}.rs`, `texture_lookup.rs`,
  `validation.rs`, `tests/cli/{common_links,models,prefox_ingame_face,textures}.rs`;
  README TC-CMN-22 (lead). Gates green (`.tmp/4_y/gates_sub_b3b2_r2.log`, acceptance 317
  of 319, the two open ones (b4)'s); mutants r0 82 mutants, 61 caught, 14 unviable, 7
  missed (`.tmp/4_y/mutants_sub_b3b2_r0/`), after the reworks 81 mutants, 67 caught, 14 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b2_r2.log`).
  (b3b3) done 2026-10-10 (SWE-2 High, the trial's second slice; brief
  `.tmp/4_y/brief_sub_b3b3.md`, continue `_c1` after a Devin CLI crash, rework `_r1`,
  reports beside them): `processing/texture.rs` `folder_textures`/`resolve_path` let a
  path the player's own folder holds, as a texture or a texture link standing there, win
  over every combined folder's copy whatever the packages, the combined copy left out and
  `shared_texture_overridden` (I, `Keep`) noted when the bytes differ or his is a link;
  `shared_texture_conflict` left to two combined folders of different packages;
  `merged_texture_conflict` removed from `messages.rs` (withdrawn in `messages.md`). The
  writer's `commit_folder` no longer takes an empty textures batch for a failed one: a
  batch whose every path a link of the player's won carries only `Keep` findings and its
  packages commit (a gap of the brief, caught by the sidekick). Help text rewritten.
  TC-MOD-04 and TC-MOD-33's tests rewritten to the new scenarios, TC-TEX-16 proven; the
  uncited `under_ingame_face_..._is_his` test (a TC-MOD-04 variant) left for 4.y-scen.
  Rework 1: the mutation run's two survivors (two combined folders holding identical
  bytes stay silent, a test added; a dead guard on the link arm removed, a shared
  folder's link having no role) and three stale doc comments. Files: `team_compiler`
  `help/01_command_line.md`, `messages.rs`, `output/writer.rs`, `plan/mod.rs`,
  `processing/{mod,texture}.rs`, `tests/cli/{models,textures}.rs`. Gates green
  (`.tmp/4_y/gates_sub_b3b3_r1.log`, acceptance 318 of 320, the two open ones (b4)'s);
  mutants r0 21 mutants, 16 caught, 3 unviable, 2 missed (`.tmp/4_y/mutants_sub_b3b3_r0/`),
  after the rework 20 mutants, 16 caught, 4 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b3_r1.log`).
  (b3b4a) done 2026-10-10 (SWE-2 High; brief `.tmp/4_y/brief_sub_b3b4a.md`, continue
  `_c1` after a Devin server incident, rework `_r1`, reports beside them): on PES 15-17 a
  `Common/` `.mtl` and a converted Common FMDL's set resolve their textures as a model
  folder's `.mtl` does, `Common/` their model folder (`prefox_face::FolderPlaces::common`
  over every kept Common texture at its path, `roles::common_folder`): a name nearest
  first from the `.mtl`'s directory up to `Common/`, a `./sub/` path at that path, each
  texture named at its own path in the team's Common output. The Common models task
  (`prefox_common`, planning handing it the Common textures' paths instead of one
  directory's stems) and the `ingame_face` boots and gloves copy (`prefox_shared`, which
  no longer consults the player's folders for a Common `.mtl`, as pipeline step 4 says)
  share it; the deep pass looks a Common `.mtl`'s `./sub/` path up at that path
  (`deep::materials::held_common`). `skeleton_beside` removed (`common_skeleton` takes
  the model's path); `MaterialPlaces::below` no longer optional. TC-CMN-23 and TC-CMN-24
  (lead) proven. Files: `team_compiler` `deep/{materials,mod}.rs`, `plan/{mod,roles}.rs`,
  `processing/{mod,prefox_common,prefox_face,prefox_shared}.rs`, `texture_lookup.rs`,
  `tests/cli/{common_links,prefox_ingame_face}.rs`; README TC-CMN-23, TC-CMN-24. Gates
  green on the VPS (`just gates-remote`, `.tmp/4_y/gates_sub_b3b4a_r1.log`, acceptance
  320 of 322); mutants r0 15 mutants, 10 caught, 5 unviable, 0 missed
  (`.tmp/4_y/mutants_sub_b3b4a_r0/`), after the rework 18 mutants, 11 caught, 7 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b4a_r1.log`).
  (b3b4b) done 2026-10-10 (SWE-2 High; brief `.tmp/4_y/brief_sub_b3b4b.md`, reworks `_r1`,
  `_r2`, reports beside them; plan change `e45f37c`, DECISIONS "on Fox a linked Common model's
  directory's textures travel with it"): `roles::linked_common_files` brings every
  texture directly in a linked model's or `.mtl`'s `Common/` subfolder (packed, not
  `file_not_used`), and only a link that takes a role reads its target (S18.2-3); a Fox
  Common part's stems resolve in one `TextureFolders` over the Common textures Fox packs
  (`processing/model.rs` `common_folders`, `common_folder()` the model folder, nearest
  first from the part's file, `./sub/` at its path), the stems the folder's texture links
  stand for inserted as links (the sidekick's contradiction, accepted: an installed CPK's
  texture link must stay found); a pre-Fox Common path into a subfolder is resolved there
  too, not kept as written (S18.2-1); the deep pass checks a Fox Common `.mtl` from its own
  directory (`held_common`), `TextureSources::below` no longer optional;
  `aesthetics_export` checks stem conflicts per `Common/` directory (S18.2-2: two textures
  of one stem in a subfolder were a run-fatal duplicate CPK path). TC-CMN-25 (lead)
  proven. Files: `aesthetics_export` `validate/{folders,tests}.rs`; `team_compiler`
  `deep/{materials,mod}.rs`, `plan/{mod,roles}.rs`, `processing/{model,prefox_face}.rs`,
  `texture_lookup.rs`, `validation.rs`, `tests/cli/common_links.rs`; README TC-CMN-25
  (its THEN reworded: the linked `body.fmdl` packs under the face fallback name
  `fcl_hair.fmdl`, so "the model built from body.fmdl"). Gates green on the VPS
  (`.tmp/4_y/gates_sub_b3b4b_r2.log`); mutants r0 18 mutants, 15 caught, 3 unviable,
  0 missed (`.tmp/4_y/mutants_sub_b3b4b_r0/`); r1 46 mutants, 35 caught, 8 unviable, 3
  missed (`.tmp/4_y/mutants_sub_b3b4b_r1/`: the deep pass's Fox Common subfolder path
  untested end to end, two tests added; an always-true guard dropped; the engine
  decided by one `match`); after r2 44 mutants, 36 caught, 8 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b4b_r2.log`).
  (b3b4c) done 2026-10-11 (SWE-2 High; brief `.tmp/4_y/brief_sub_b3b4c.md`, reworks `_r1`, `_r2`,
  reports beside them; plan change `bfe0adb`, DECISIONS "a linked Common file brings the
  textures above it too"): `linked_common_files` brings the textures directly in each
  directory above a linked model's or `.mtl`'s up to `Common/` (S18.A3-1); the Fox
  `CommonSet` fallback resolves from the Common model's own path, not the player's `.mtl`'s
  (S18.A3-2), and the deep pass falls back the same way on Fox (the sidekick's
  contradiction, accepted; pre-Fox has no such fallback in the task either; a player's
  `.mtl` over his own model still never resolves in `Common/`, r2's test); on Fox every
  pre-Fox Common texture path is pointed at Fox's Common output at its subdirectory, held or
  not, `point_texture` returning its supply for `texture_supply` (S18.A3-3: an
  installed-supplied one kept its pre-Fox directory). Ruled test changes: three
  (`processing/model.rs` two, `tests/cli/conversion.rs` one, which now reports the
  unsupplied Common path's `fmdl_texture_not_found` beside the deep pass's
  `mtl_texture_not_found`). Files: `team_compiler` `deep/{mod,pairings}.rs`,
  `plan/roles.rs`, `processing/model.rs`, `tests/cli/{common_links,conversion}.rs`. Gates
  green, locally again (`.tmp/4_y/gates_sub_b3b4c_r2.log`); mutants r0 23 mutants, 20
  caught, 3 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b4c_r0/`); r1 29 mutants, 24 caught,
  3 unviable, 2 missed (the deep fallback's pairing guard untested, `mutants_sub_b3b4c_r1/`);
  after r2 29 mutants, 26 caught, 3 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b4c_r2.log`).
  Open: whether a player's `.mtl` over a Common `.model` on PES 15-17 needs the same Common
  fallback (`prefox_face::face` resolves it in the folder alone; `pipeline.md` step 3 says a
  player's material files layer over a Common-linked model's).
  (b3c1) done 2026-10-11 (SWE-2 High, one run and one rework round; briefs
  `.tmp/4_y/brief_sub_b3c1{,_r1}.md`, reports beside them;
  plan `243fa9a`, refined by S19's `bcd400b` and `bb0aa44`): a shared folder's tree follows
  the player folder's rules. AE `check_shared` admits below a subfolder what a player's
  subfolder admits but links (its `.common` links are (b3c2)'s), checks stems per directory
  through one helper shared with `check_player` (`tree_stem_conflicts`), and applies the
  boots/gloves suffix rule at any depth, naming the path below the folder; TC `role_position`
  drops its `shared` flag (one rule for both folder kinds, the singletons the root's alone),
  the combined texture places are keyed per directory as the folder's own are, each caller
  passing the file's own source folder (`Part::source`, `FolderPlaces::of(source, path)`),
  and a PES 15-17 shared gloves output packs a subfolder's models and `.mtl` files at their
  paths, `glove.xml` listing `./grip/glove_r.model`, each `.mtl` naming its texture relative
  to itself (`TextureDirectory::of_below`: `./`, `../`, `./<sub>/`); `boots.mtl`, merged at
  the root, names a subfolder texture `./<sub>/`. Fox's shared output already put a
  subfolder texture at `k{id}/<sub>/#windx11/` (S19.2-B2). Ruled test changes: six
  (`plan/roles.rs` two, `texture_lookup.rs` one, `user_face_xml.rs` one, `tests/cli/models.rs`
  TC-MOD-68 and the nested-boots test, now `fmdl_name_invalid`). Files: AE
  `validate/{folders,tests}.rs`; TC `deep/{materials,mod}.rs`, `paths.rs`, `plan/{mod,roles}.rs`,
  `processing/{model,prefox_common,prefox_face,prefox_shared,texture}.rs`,
  `texture_lookup.rs`, `user_face_xml.rs`, `validation.rs`,
  `tests/cli/{models,prefox_faces,textures}.rs`. Gates green locally
  (`.tmp/4_y/gates_sub_b3c1_r1.log`); mutants r0 116 mutants, 94 caught, 21 unviable, 1
  missed (the `linked_local` filter, `.tmp/4_y/mutants_sub_b3c1_r0/`); after r1's test 116,
  95 caught, 21 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3c1_r1/`).
  (b3b4d) done 2026-10-11 (SWE-2 High, one run; brief `.tmp/4_y/brief_sub_b3b4d.md`,
  report beside it): a texture path into a subdirectory of the team's Common output is
  looked up at that subdirectory on both engines, where three places called it supplied
  unlooked. Fox `texture_supply` reads the subdirectory from any `common_home` spelling
  (`TextureDirectory::subdirectory`) and asks one helper, `common_below_supply`, that
  `point_texture` asks too (the Common textures packed there, link stand-ins included,
  then the installed CPKs); a path the pointing resolved records its verdict, so a
  player's own home is looked up only when nothing pointed it there (an earlier CPK's, as
  `pipeline.md` says). The deep pass builds `common_below` on both engines
  (`KeptCommon::texture_belows`), and its per-folder Common places gain the folder's
  texture-link stand-ins as the Models task's have (S18.3-1); pre-Fox
  `common_texture_stems` keeps subfolder stems. A `kitN/` subdirectory is left unlooked by
  one guard, `kit_variants::kit_reference_directory`, until (b4); in `point_texture` and the
  deep pass, which reported it missing before, that is the sidekick's accepted
  contradiction (one guard; a `kitN` directory stands for its set). S18.3-4, S18.3-5 done.
  Ruled test changes: three (`installed.rs`'s pre-Fox stems, `deep/materials.rs`'s
  `refkit/` path now missing, `processing/model.rs`'s player-home path now looked up). Files:
  TC `bins/installed.rs`, `deep/{materials,mod}.rs`, `kit_variants.rs`, `paths.rs`,
  `plan/mod.rs`, `processing/model.rs`, `tests/cli/{prefox_textures,textures}.rs`. Gates
  green locally (`.tmp/4_y/gates_sub_b3b4d.log`); mutants 25 (13 here, 12 on bonfire), 20
  caught, 5 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3b4d/`).
  (b3c2a) done 2026-10-11 (SWE-2 High, one run and two rework rounds; briefs
  `.tmp/4_y/brief_sub_b3c2a{,_r1,_r2}.md`, reports beside them; Astra rounds A1 6/6 and
  A2 3/4 accepted, `.tmp/4_y/duck_rulings.md` B3C2A.A1-A2): a shared folder's `.common`
  links at any depth. AE `check_shared` admits them (a shared link stays disallowed), and on
  PES 18-21 resolves and checks them as `check_player` does (`links::common_link_missing`,
  one helper for both), its cascade dropping a shared folder whose link's target a pass
  dropped and its linkers in the same pass, a kept folder losing a link with an absent
  target (`links::without_dropped`, one filter for both folder kinds); a texture link claims
  its stem per directory on Fox, nothing on pre-Fox, where every shared-folder link is
  `file_not_used`. TC: `fox_file` gives a shared folder's texture link its role (model and
  `.mtl` links wait for (b3c2b)); `linked_common_files` counts shared folders' links; a
  shared output's `ModelFolder::common_files` holds what its links reach
  (`roles::shared_common_files`); the deep pass's Common places read every combined
  source's links; a combined folder's texture link takes part in the combined-conflict
  rule with its folder's package (`processing/texture.rs` `LinkCopy`, `resolve_path`; the
  model task's lookup keeps the same winner), two links at one path naming one `Common/`
  texture; `output/writer.rs` `commit_folder` keeps the packages of a batch left empty by a
  conflict the links won. Help topic `01_command_line.md` gains shared folders' links.
  Plan: TC-MOD-75 to TC-MOD-78, TC-TEX-17, TC-TEX-18; `shared_texture_conflict` and
  `shared_texture_overridden` rows (a link counts as differing; two links report nothing).
  Files: AE `validate/{folders,links,mod,tests}.rs`; TC `deep/{mod,pairings}.rs`,
  `mtl_search.rs`, `output/writer.rs`, `plan/{mod,roles}.rs`,
  `processing/{model,texture}.rs`, `validation.rs`, `help/01_command_line.md`,
  `tests/cli/{common_links,models,prefox_faces,textures}.rs`. Gates green locally
  (`.tmp/4_y/gates_sub_b3c2a_r2.log`); mutants r0 69 (3 missed,
  `.tmp/4_y/mutants_sub_b3c2a_r0/`), r1 90 (4 missed, `_r1/`), r2 92: 79 caught, 12 unviable,
  1 missed (`_r2/`: the pre-Fox claim's texture test in `shared_texture_claim`), killed by the
  lead's added assertion (two textures of one stem still collide on PES 17), red with the
  mutant applied by hand.
  (b3c2b) done 2026-10-11 (SWE-2 High, one run and one rework; briefs
  `.tmp/4_y/brief_sub_b3c2b{,_r1}.md`, reports beside them): on PES 18-21 a shared folder's model and `.mtl` links take a player folder's
  roles (`fox_file`, `FolderModels` lifted their shared-folder guards; pre-Fox unchanged). A
  shared boots or gloves folder holding only a model link of its kind takes an id
  (`ids::holds_model_of_its_kind`); a shared output's `ModelFolder` gets the `Common/` files
  every player folder gets on Fox (`roles::shared_common_files` removed) and its own Common
  models; `common_models` resolves every source's links through `part_source_models`, a
  combined source's `.model` link searched in its own folder; `ModelFolder::roles` gives one
  `Common/` file one part per folder, the player's own link first; the Fox Models task's
  `.model` search reaches `Common/` for a combined source; the deep pass pairs a shared
  folder's links against `Common/` and `searched_common_mtls` searches shared folders too;
  `drop_gltf_folders` drops a player whose linked shared folder's link names a model a Common
  glTF beats. Contradictions: TC-MOD-80's THEN (a face-folder `body` link is
  `fmdl_fcl_hair_fallback` as a player's, scenario reworded) and the pre-Fox combined search
  kept at no `Common/` file, both accepted. Plan: TC-MOD-79 to TC-MOD-82; `messages.md`
  `shared_folder_no_model`; help topic `01_command_line.md`. Files: TC `deep/{mod,pairings}.rs`,
  `mtl_search.rs`, `plan/{ids,mod,roles}.rs`, `processing/model.rs`, `validation.rs`,
  `tests/cli/common_links.rs`. Gates green locally (`.tmp/4_y/gates_sub_b3c2b_r1.log`); mutants r0 43 (4 missed: three in dead pre-Fox arms of the Fox-only Models task, removed; the pre-Fox shared search's guard, tested; `.tmp/4_y/mutants_sub_b3c2b_r0/`), r1 40: 31 caught, 9 unviable, 0 missed (`.tmp/4_y/mutants_sub_b3c2b_r1.log`).
  (b4a) done 2026-10-11 (SWE-2 High, one run and three reworks, the first finished by the
  lead after its session ended twice at the model's output limit, the next two on a fresh
  session; briefs `.tmp/4_y/brief_sub_b4a{,_r1,_r2,_r3}.md`, reports beside them; Astra
  rounds B4A.A1 12 concerns, 10 accepted, A2 9, 6 accepted, A3 3, 1 accepted). `aesthetics_export`
  `KitToken::Fallback` (`kitF`, spelled exactly so). `team_compiler`: `kit_variants`'
  `fallback` (the `kitF` member, else the lowest variant) and `representative` (the lowest
  variant, else `kitF`), `kit_lookup_key` filing a `kitF` file under its set's `kitN` key in
  every by-stem lookup, the token read in the file name alone (a prefix token below a folder,
  a subfolder's sets); `complete_kit_variants` writes Fox's fallback once as `kitN` and
  copies pre-Fox's into each gap, `kit_variant_missing` only for a variant fallback; model
  sets use `kitF`; a member's `face.xml` `kitN` resolves to the lowest variant for
  validation; `Common/`'s direct texture stems and the links' stand-ins kept as spelled. The
  PES 15-17 face reads tokens from source stems, copies a set's gaps from its fallback under
  the listed entry's names, and strips `kitF` files once where `face()` returns
  (`packed_fallback_stem`: the packed name is lower-cased, so an ordinary file spelled `kitf`
  is stripped too, accepted). A separate store for the fallback's files (round 1's ruling
  A1-9) was reversed at round 2: it had to repeat the package's collision check, the hand
  split's material lookup and the hand-off to a shared face's `face.xml`, and missed all
  three. Files: AE `conventions/kit_token.rs`; TC `deep/{materials,mod}.rs`,
  `kit_variants.rs`, `plan/{mod,roles}.rs`, `processing/{mod,model,prefox_face,texture}.rs`,
  `texture_lookup.rs`, `user_face_xml.rs`, help `01_command_line.md`,
  `tests/cli/{common_links,models,prefox_faces,prefox_hand_split,prefox_kit_variants,prefox_user_xml,textures}.rs`.
  Gates green locally (`.tmp/4_y/gates_sub_b4a_r3.log`); mutants on bonfire: r0 89, 0 missed (`.tmp/4_y/mutants_sub_b4a/`); r1 112, 4 missed (`_r1/`: `set_completions` gathering unlisted variants, which never are the fallback: arm removed; two sets in one face, tested); r2 121, 6 missed (`_r2/`: those four and `packed_fallback_stem`'s delimiters, tested); r3 118: 106 caught, 12 unviable, 0 missed (`_r3/`).
  (b4b1) done 2026-10-11 (SWE-2 High, one run and two reworks on one session, 19, 7 and 3 minutes; Astra rounds B4B1.A1 8 concerns, 8 accepted, A2 3, 2 accepted, 1 deferred to (b4b2), both on the Devin Pro `brother` slot; briefs
  `.tmp/4_y/brief_sub_b4b1{,_r1}.md`, reports beside them). `aesthetics_export`:
  `kit_set_member` (a path's member token in the set a `kitN` stem or directory segment
  names) and `names_kit_fallback` (`conventions/kit_token.rs`); `common_link_files`
  (`validate/links.rs`), the one link rule validation and the planner share, texture
  targets only: the brief's "every caller of `common_file` gets the set's representative"
  made a model link's set reachable, and the per-kit model pipeline does not serve Common
  models (round 1: a panic, the wrong base model, a lost `.skl`). `team_compiler`:
  `roles::common_file` is its first file, `linked_common_of` links every member. Files: AE
  `conventions/{kit_token,mod}.rs`, `lib.rs`, `validate/{links,mod,tests}.rs`; TC
  `plan/roles.rs`, `tests/cli/common_links.rs`. Gates green locally (`.tmp/4_y/gates_sub_b4b1_r2.log`); mutants on bonfire, each round 0 missed: r0 57 (45 caught, 12 unviable), r1 58 (45, 13), r2 57 (45, 12) (`.tmp/4_y/mutants_sub_b4b1{,_r1,_r2}/`).
  (b4b2a) done 2026-10-11 (SWE-2 High, one run and one rework on one session, 23 and 12 minutes; Astra round B4B2A.A1 3 concerns, 3 accepted, on the Devin Pro `home` slot; briefs
  `.tmp/4_y/brief_sub_b4b2a{,_r1}.md`, reports beside them). `aesthetics_export`:
  `kit_path_token` and `variant_path` (`conventions/kit_token.rs`), one reading of a path's
  token, the file name's first, else the first directory segment spelled exactly as one
  (`kit_folder`); `kit_set_member` and `names_kit_fallback` rebuilt on it, so validation's
  link rule and the compiler's sets agree. `team_compiler`: `complete_kit_variants` reads
  the folder form (Fox's fallback once as `kitN/armor_bsm`, pre-Fox's copies into
  `kit3/armor_bsm`); `plan::texture_directory` gives a `Common/` texture the directory above
  its first kit-token folder, one `CommonTextures` task per such directory (Common models
  stay one task per directory, models having no folder form), each texture keyed by its
  path below the task's folder; test mode keeps that path (`pipeline.md` step 5, DECISIONS
  2026-10-11), and a conversion finding names it. Files: AE `conventions/{kit_token,mod}.rs`,
  `lib.rs`; TC `paths.rs` (`texture_below`, `common_texture_below`: test mode reads a
  texture's path below its folder back from its game path, the output target staying
  decided at the materialize seam), `plan/mod.rs`,
  `processing/{materialize,mod,referee_marker,texture}.rs`,
  `tests/cli/{common_links,test_mode}.rs`; plan `team_compiler/pipeline.md`. Gates green locally (`.tmp/4_y/gates_sub_b4b2a_r1.log`); mutants on bonfire, each round 0 missed: r0 51 (43 caught, 8 unviable), r1 66 (58, 8) (`.tmp/4_y/mutants_sub_b4b2a{,_r1}/`).
  (b4b2b1) done 2026-10-11 (SWE-2 High, one run, 26 minutes; no Astra round of its own:
  reviewed with (b4b2b2) in one round on the Devin `work` slot; brief
  `.tmp/4_y/brief_sub_b4b2b1.md`, report `.tmp/4_y/brief_sub_b4b2b1.notes.md`).
  `team_compiler`: `kit_variants::kit_lookup_keys` replaces `kit_lookup_key` (a variant
  under its own folded path and its set's `kitN` reference, a `kitF` member under the
  reference alone), and every texture key set files by it: `TextureFolders::place` (the
  reference key's value the reference's spelling, so a path naming the set is pointed at the
  `kitN/` directory the output writes), the deep pass's Common sets, an installed CPK's
  Common stems. Every lookup is then a plain folded match: `has_variant_among`,
  `texture_lookup::variant` and the `kit_reference_directory` guard retired. Files:
  `bins/installed.rs`, `deep/{materials,mod}.rs`, `kit_variants.rs`, `plan/mod.rs`,
  `processing/{model,prefox_face}.rs`, `texture_lookup.rs`, `tests/cli/textures.rs`. Gates green locally (`.tmp/4_y/gates_sub_b4b2b1_r0.log`); mutants on bonfire 26, 22 caught, 0 missed, 4 unviable (`.tmp/4_y/mutants_sub_b4b2b1_r0/`).
  (b4b2b2) done 2026-10-11 (SWE-2 High, one run and one rework on one session, 13 and 4 minutes; Astra round B4B2B.A1 over (b4b2b1) and (b4b2b2), 3 concerns, 1 accepted, 1 deferred, 1 rejected, on the Devin `work` slot; briefs
  `.tmp/4_y/brief_sub_b4b2b2{,_r1}.md`, reports beside them). `aesthetics_export`:
  `validate/links.rs` `common_link_dropped`, the cascade's one test of a `.common` link's
  target: every `Common/` file `common_link_files` reaches is dropped, so a kit set falls
  only with its last member (plan `object_model.md` "Dropped link targets", DECISIONS
  2026-10-11). `team_compiler`: (b4b2b1)'s leftovers, an `expect` for an unreachable
  fallback in `TextureFolders::place` and the test helpers building their places through it;
  `installed_lookup` one keyed lookup, `InstalledPaths::holds` gone (Astra B4B2B.A1-3).
  Files: AE `validate/{links,tests}.rs`; TC `bins/installed.rs`, `deep/materials.rs`,
  `processing/model.rs`, `texture_lookup.rs`, `tests/cli/textures.rs`; plan
  `aesthetics_export/object_model.md`, `team_compiler/README.md` (TC-CMN-31). Gates green locally (`.tmp/4_y/gates_sub_b4b2b2_r1.log`); mutants on bonfire, each round 0 missed: r0 6 (6 caught), r1 6 (6) (`.tmp/4_y/mutants_sub_b4b2b2{,_r1}/`).
  Verify 2026-10-11 (the lead): 341 of 341 scenarios proven, TC-MOD-65 to TC-MOD-68 and
  TC-REF-18 among them. The census export `HAHA Aesthetic Export VTL9 i3` is in the old
  layout (`export_layout_old`; the Export upgrader is Phase 6's), so a hand-made new-layout
  copy of its players and `Common/` (`.tmp/4_y/census/migrate_haha.py`: `Faces/xxxNN - Name`
  as `Players/NN - Name`, kits and logo left out; 23 players, 688 nested files) went through
  `check` and `compile --mode test` for PES 17 with a release build of `6b9cc30`, the census
  binary's teams list naming slot 853 `/HAHA/` as the cup does for an invitational
  (`.tmp/4_y/census/logs_17_haha_new/`): 22 of 23 players kept, each with its subfolder
  tree in the output (772 files); `01 - SELENBEAR` is dropped by a real authoring defect,
  `model_material_undefined` on `body/oral_hand_r.model`, whose second mesh binds a material
  with an empty name its `.mtl` does not define (read with `pes_model`; the empty name prints
  as `materials=`); the rest are Warnings: seven `mtl_texture_not_found` for the kit logo
  `teamlogo_u0853p0.dds`, which the old layout's `Common/teamlogo_u0XXXp0.dds` stands for
  and only the Export upgrader renames, two for textures a `.mtl` names that its folder
  lacks, and five `xml_model_unlisted`. Face CPKs were not built (test mode writes the tree
  loose).

- [x] 4.y-kitwarn **A sleeve or fit option the shirt model ignores is one Warning** (the
  maintainer's answer, 2026-10-10; DECISIONS 2026-10-10). The deep pass maps `kit_config`'s
  three `kit_*_requires_model_144_or_160` findings to `kit_config_option_ignored` (W, kept,
  context the file, the option and the shirt model), as it maps the two clamps to
  `kit_config_version_clamped`. Plan: `pipeline.md` "Kit configs", `messages.md` row.
  Crates: tc
  → verify: a CLI test per option (cut-out short sleeves, undershirt-only long sleeves, the
  tight fit, each on a shirt model outside 144 and 160), red first; a config on model 144
  with the three options reports nothing.
  Done 2026-10-10 (Opus 5.5, one run and one rework round): `Code::KitConfigOptionIgnored`
  (Warning); `deep/documents.rs` `option_ignored` maps the three `validate` findings to the
  option's TOML key (`shirt.short_sleeves`, `shirt.long_sleeves`, `shirt.tight`) and the
  shirt model, reported by `kit_config_findings` after the clamps, on the file, kept. The
  rework, the sidekick's contradiction: the config is checked as emitted, the FPC values
  applied first when the team's kit-FPC status is On (`EffectiveTeamKitFpc::of`, now
  `pub(crate)`), since `apply_fpc` sets the shirt model to 176 and a config written for
  144 or 160 lost its option in game with no finding (DECISIONS 2026-10-10 "checks the
  config as emitted"; pipeline.md, messages.md; the status read before the deep pass's
  own drops, a plan note). `bins/kit_configs.rs`'s installed-config
  patch is untouched (it changes no sleeve or fit value). Help paragraph in
  `01_command_line.md`. Tests: TC-KIT-32 (`compile_exports.rs`: `check` and `compile`
  for PES 21, three Warnings, nothing on model 144, p1's config emitted with its cut-out
  sleeves), the `fpc_on` variant (model 144 reported as 176), the `documents.rs` unit
  test (order after the clamps, the FPC case). Files: `team_compiler` `messages.rs`,
  `deep/documents.rs`, `deep/mod.rs`, `plan/mod.rs`, `help/01_command_line.md`,
  `tests/cli/compile_exports.rs`; README TC-KIT-32, pipeline.md, messages.md, DECISIONS
  (lead). Gates green (`.tmp/4_y/gates_kitwarn_r2.log`, acceptance 309 of 314); mutants
  13/0/1 (`.tmp/4_y/mutants_kitwarn_r2.log`).

- [x] 4.y-retag **The 2026-10-05 retags reviewed** (the maintainer's answer, 2026-10-10:
  "whichever fits Conventional Commits best"; rebasing and force-pushing are fine, the
  repository being unannounced). The commits retagged from `review` to `fix` that day (the
  reviewer-round and survivor-triage commits of 2026-09-28 to 2026-10-05: `git log
  --format='%h %s' | grep -i "round\|triage\|S[0-9]"`) are re-read one by one and retagged
  `refactor` or `test` where the diff is one of those (a commit that only adds tests is
  `test`; one that moves code without changing behavior is `refactor`; one fixing a defect
  stays `fix`), with an interactive-free rebase (`git rebase` with a script `exec`, never
  `-i`) run when no sidekick edits the tree and nothing is uncommitted, then a force push.
  The same rebase cuts every subject over 120 characters (the maintainer's rule,
  2026-10-10, `CONTRIBUTING.md` "Commits"; the longest today is 368): the subject keeps
  its type, scope and the one-line gist, and what it listed moves to the body's first
  paragraph when the body does not already say it. The rebase script rewrites messages
  from a table the lead writes by hand (`.tmp/4_y/retag/messages.tsv`: hash, new subject),
  one row per commit touched, reviewed before the run.
  → verify: `git log --oneline` shows the new types and no subject over 120 characters
  (`git log --format=%s | awk 'length > 120'` prints nothing); `just gates` green at the
  new HEAD.
  Done 2026-10-10: the re-read found no commit to retype (every reviewer-round and triage
  commit changes behavior or mixes tests with fixes, so `fix` stands); the 76 subjects
  over 120 characters were cut to their type, scope and gist by `git filter-branch
  --msg-filter` from `.tmp/4_y/retag/messages.tsv` over `b197139^..HEAD`, each old subject
  kept as the body's first paragraph ("Subject as first written: ..."). 583 commits before
  and after, an empty tree diff, gates green, force-pushed (`bd5bc82`). Every hash from
  `b197139` on changed; hashes quoted in `.tmp/` notes and reports before the rewrite are
  the old ones.

- [x] 4.y-fix17 **Three small deep-pass and collar follow-ups** (brief
  `.tmp/4_y/brief_fix17.md`, 2026-10-10). (1) `held_stems` takes a marked player's combined
  boots and gloves folders (4.y-fix16's follow-up: `FaceUse` is `Unused` under the marker,
  so his combined textures did not count for his own `.mtl` paths). (2) A texture the
  decoder refuses is the deep pass's finding on the file: `texture_unreadable` (new) for a
  header it cannot read, `texture_codec_unsupported` for a codec or DDS kind it refuses,
  neither pass-through-eligible, instead of the Common textures task failing whole
  (Issues 2026-10-10; DECISIONS 2026-10-10). (3) The collar tests convert the real stock
  collar `konami_collar_001.wesys.model` (PES 17's `dt35_win.cpk`,
  `uniform/nocloth/collar_001.model`; the lead's fixture, `smoke.py`'s count 18) in place
  of the shirt fixture with a renamed material (4.y-fix12's contradiction). Plan:
  `messages.md` `texture_unreadable`, `texture_codec_unsupported`; `pipeline.md`
  "Collars". Crates: tc, pes_model (the fixture), python_bindings (the count)
  → verify: a test per item, red first; `just bindings` green with 18 model fixtures.
  Done 2026-10-10 (sidekick, one run; brief `.tmp/4_y/brief_fix17.md`, report
  `.tmp/4_y/sk_fix17_report.md`): (1) `folder_findings` takes a `ReadFolder` (path, files,
  models, the player's combined folders, the eighth argument grouped rather than a lint
  opt-out), and the `.mtl` lookup's shared folders are the linked face when the face files
  are used plus every combined boots or gloves folder, marker or not; `FaceUse::Used` lost
  its `combined` field (narrowed). The brief's link shape was wrong (a shared boots folder
  is linked as `Studs.boots`, and the link combines under the marker only beside a
  `boots.model` of his own): the test in `prefox_ingame_face.rs` uses the real one. (2)
  `deep::texture::probe_failure` maps the probe's error as `conversion_failure` does
  (`texture_codec_unsupported` for the two codec arms, the new `texture_unreadable` for the
  rest), neither pass-through-eligible; a cube map skips the probe (the sidekick's applied
  contradiction, accepted: the 2D reader refuses every cube map and the compile never decodes
  one; TC-TEX-13's `env.dds` was dropped under the brief's rule), so a cube map the Fox route
  refuses still fails its task (`messages.md`). Nine tests that fed a task junk bytes to make
  it fail now feed it `pixels_cut_dds()` (a whole header over pixel data cut short), TC-STR-09's
  empty `hair.dds` real bytes, and TC-TEX-14's finding moved from the task to the deep pass
  (TC-KIT-14's and TC-BIN-14's wording updated). (3) The collar tests convert the stock
  `konami_collar_001.wesys.model` (12 bones, two meshes on `uni_shirts` then `uni_collar`, no
  annotation; unwrapped it equals the lead's probe copy): on PES 21 only bitangents drop, no
  bone moves; on PES 15 the trapezius bones fold onto the shoulders and four retarget; PES 16
  and 17 byte for byte. Contradictions ruled: the catalog's "the file alone" was a new rule
  (the pass has no per-code disposition: the place's, as `texture_type_mismatch`), row
  reworded; `texture_codec_unsupported`'s row lists the DDS reader's other refusals; a
  PES 17 `.model` collar packed as is or moved for PES 15-16 keeps `uni_collar`, which
  their `uniform.mtl` does not define (Issues). Files: `deep/mod.rs`, `deep/texture.rs`,
  `deep/materials.rs`, `messages.rs`, `processing/texture.rs`, nine CLI test files;
  `pes_model` fixture + README row, `smoke.py` (lead); `messages.md`, `README.md` TC texts
  (lead). Gates green (`.tmp/4_y/gates_fix17_lead.log`, acceptance 302 of 312), `just
  bindings` green (37 fixtures); mutants 10 caught, 0 missed, 3 unviable
  (`.tmp/4_y/mutants_fix17.log`).

- [x] 4.y-fix18 **A `.model` collar converted for PES 18-21 takes the stock collars' Fox
  material set** (the lead's in-game check, 2026-10-10, under 4.y-ingame2). The FMDL
  4.y-fix12 writes keeps the converter's `fox3ddf_blin` with no sampler, and PES 21 draws
  nothing for it: PES 17's stock collars 1 and 50 converted under ID 12 both show the plain
  neckline a collar ID with no model shows (`c21_14_row19_neck.png`,
  `c21_24_row19_neck.png`, against the stock `collar_107.fmdl` copy's folded collar,
  `p21_19_collar_front.png`). `collar_samplers_dropped` becomes the stock material set by
  name: `uni_collar` gets shader `pes_3ddf_collar`, technique `pes3DDF_Collar_NC`;
  `uni_shirts` `pes_3ddf_shirt_nb`, `pes3DDF_Shirt_NB_NC`; each binds the one sampler
  `Pattern_Tex_LIN` to `/Assets/pes16/model/character/common/sourceimages/uni_pattern.dds`
  and carries the stock params (`MatParamIndex_0 [40]`, `BlendNormalXParam 0`,
  `BlendNormalYParam 0.666`, `RepetitionParam 80`, `BlendCoeffParam 0`, `BlendBoostParam 0`,
  `PatchAnisoRoughnessParam 0.4`, `PatternIndexParam 1`; the sidekick's measure of the stock
  `collar_107.fmdl`, `.tmp/4_y/sk_fix12/probe.txt`), meshes alpha 32, shadow 0; any other
  material name is `model_conversion_failed` (a collar names those two alone). The help
  text's "name no texture" sentence changes. If the stock FMDL differs from the converted
  one past the materials (bones, mesh flags, vertex layout: compare with `fmdl`'s reader
  and report), the brief grows. Plan: `pipeline.md` "Collars" (the Fox material set
  paragraph); DECISIONS 2026-10-10. TC-CMN-11 (its materials assertion changes). Crates: tc
  → verify: TC-CMN-11 proven with the stock set, red first; then the lead's PES 21 run of
  `.tmp/4_y/ingame/build_collar21_export.py` shows a collar on row 19.
  Done 2026-10-10 (Opus 5.5, one run): `stock_collar_fox_materials` (a `match` on the
  material name, the eight parameters a `const`, `anyhow` as `failed` takes), run before
  the `fmdl` check, every mesh set to alpha 32 and shadow 0 (the sidekick's catch, applied:
  the brief read the one-mesh probe, but `uniform.mtl`'s `uni_collar` is one-sided, so
  the fixture's second mesh converted at alpha 0); TC-CMN-11 asserts both materials in
  full in the `.model`'s order and the mesh flags, red first; a unit test refuses a collar
  naming `skin_limb`. The stock `collar_107.fmdl` against the converted one past the
  materials (`.tmp/4_y/sk_fix18_report.md` (4)): the stock carries a vertex colour stream
  (a grey ramp, 64 to 255), the converted none (the `.model` has none); 16 bones to 12
  (`sk_belly`, the deltoids and an underarm bone, weighted by no vertex), every stock
  `dsk_*` a root at `sk_belly`; eight levels of detail to one; the same region. In game
  (4.y-ingame2's manual line): the collar draws, so none of these matters. Files:
  `processing/conversion.rs`, `processing/mod.rs`, `help/01_command_line.md`,
  `tests/cli/collars.rs`. Gates green (`.tmp/4_y/gates_fix18.log`, acceptance 302 of
  312); mutants 1 caught, 0 missed, 3 unviable (`.tmp/4_y/mutants_fix18.log`; the
  unviables are whole-function replacements of types with no `Default`).

- [x] 4.y-fix19 **S9's third Astra round: the deep pass reads every Common `.mtl` a link's
  search may fall back to, and a non-model file in `Collars/` gets no collar finding**
  (rulings S9.A3 in `.tmp/4_y/duck_rulings.md`; two of three accepted, so the S9 Astra loop
  ends with this rework). (1) `deep/mod.rs` `searched_common_mtls`: when a link's search
  finds its `.mtl` in `Common/`, the set takes every direct `Common/` `.mtl`, since the
  search's chain there ends with any `.mtl` in name order and a dropped first find leaves
  the next one unread (its own comment said so); `model_material_undefined` then fires on
  the fallback. (2) `deep/collar.rs` `collar_findings`: the file's kind is matched before
  the stem is read as a collar ID, so a `Collars/README.txt` the lenient structure pass
  keeps is passed over as planning passes it over, not `collar_id_invalid`. Plan:
  `pipeline.md` "Deep format pass" (unchanged: the pass checks what `compile` reads);
  `messages.md` `collar_id_invalid`. Crates: tc → verify: a test per item, red first.
  Done 2026-10-10 (Opus 5.5, one run with 4.y-fix15; the step was ruled with `f8de614`,
  whose edit script lost its insertion). (1) `searched_common_mtls` returns every direct
  `Common/` `.mtl` once any player's search lands on a `Common/` file, none otherwise; the
  cost, accepted: a broken Common `.mtl` nothing reads is `mtl_broken` too, one rule over a
  per-candidate chain. Test: `common_links.rs`
  `a_common_mtl_a_link_s_search_falls_back_to_is_checked_on_pes_21` (a broken `legs.mtl`
  beside a `materials.mtl` with the card's material renamed: `mtl_broken` on the one,
  `model_material_undefined` naming the other, exit 1; red: `mtl_broken` alone and the
  folder planned). (2) `collar_findings` matches the kind first: a non-model kind gets no
  finding; a glTF is a model whose name is checked and whose content is not, as before,
  since `check` reports nothing else of it (the brief's two readings, the sidekick's kept;
  a `neck.glb` row pins it in the unit test). Test: `collars.rs`
  `a_file_of_no_model_kind_in_collars_kept_by_the_lenient_check_gets_no_collar_finding`
  (`Info file_type_disallowed` alone, exit 0, the collar left out; red: `collar_id_invalid`
  and exit 1). Mutants: one survivor, the set filter's `&& is_direct_root_folder_file`,
  equivalent (the caller asks the set about a direct `Common/` file alone), the redundant
  half dropped by the lead with the doc saying why; the rework test the lead asked for
  against the wrong `&&` (`a_common_mtl_no_link_names_is_not_checked_when_a_model_pairs_locally_on_pes_21`)
  stays as the direct proof of the "none otherwise" branch, the sidekick having caught the
  misreading. A Common `.mtl` nothing links gets no finding at all today (seen there; not
  this step's). Files: `deep/mod.rs`, `deep/collar.rs`, `tests/cli/common_links.rs`,
  `collars.rs`. Gates and mutants: with 4.y-fix15's.

- [x] 4.y-fix20 **S17's Astra round: the hidden-model drop moves to the deep pass, a
  package's names collide case-folded, the hand split's test proves every face packed
  once** (rulings S17.A1 in `.tmp/4_y/duck_rulings.md`; four of four accepted, under
  five, so the S17 Astra loop ends with this rework; DECISIONS 2026-10-10 "dropped by the
  deep pass"). (1) On PES 15-17 the deep pass drops an FMDL whose every mesh is hidden
  with the Info `model_hidden_dropped` (on the file, `DropFile`, not pass-through-eligible,
  both commands), a `.common` link naming it with the same Info on the link (a drop rides
  on a finding), so no task converts or names it: the face's `in_common` entry for an all-hidden Common model named a
  `.model` the Common task never wrote, and a shared boots folder whose every FMDL was
  all-hidden reached `merge` with no part (`folder_pack_failed`). `fmdl_for_pre_fox`
  returns the conversion, `EveryMeshHidden` a failure like any other, and the `None`
  plumbing in the face, shared, Common and collar tasks goes; a planning finding that
  follows from the drop (a shared folder left with no model) stands as it is and is
  reported in the brief's closing section. (2) `prefox_face.rs` `insert` refuses a name
  already in the package case-folded (`vtree::fold_name`). (3) `tests/cli/prefox_hand_split.rs`
  (~104) and `tests/cli/conversion.rs` (~1065) compare the source's oriented-face multiset
  against the union read back from the three packaged models, not counts. Plan:
  `pipeline.md` "Deep format pass", `messages.md` `model_hidden_dropped`, `ir.md`.
  Crates: tc
  → verify: TC-MOD-69 proven through the deep pass (`check` reports it too); a test per
  item, red first: a player linking an all-hidden `Common/oral.fmdl` compiled for PES 17
  gets a face naming nothing of it; a shared boots folder holding one all-hidden FMDL
  compiled for PES 17 fails no task; `Face_High.fmdl` beside `face_high.mtl` is the
  insert error; the hand split test fails when a hand's faces are packed twice.
  Done 2026-10-10 (Opus 5.5, one run and one rework round; brief
  `.tmp/4_y/brief_fix20.md`, report `.tmp/4_y/sk_fix20_report.md`). (1)
  `model_convert::formats::fmdl::every_mesh_hidden`
  is the `.model` export's rule (every mesh carries the `invisible` shadow bit; a model with
  no mesh is not hidden), tied to `ir_to_model`'s `EveryMeshHidden` by a six-model test;
  `deep/model.rs` `ModelRead.hidden`; `deep/mod.rs` `file_outcome` emits
  `model_hidden_dropped` (Info, `DropFile`, not eligible, context `file`) on a PES 15-17 run
  for a player, shared, `Common/` or `Collars/` FMDL, `ContentPass.hidden_models`, and
  `hidden_link_findings` puts the same finding on a player's `.common` link to a dropped
  Common FMDL (context `file` the link, `model` the export path); `fmdl_for_pre_fox` returns
  the conversion, the `Option` plumbing gone from the face, shared, Common and collar tasks.
  The sidekick's contradiction, applied and accepted (DECISIONS 2026-10-10 "a `DropFile`
  finding on a player or shared folder's own file takes the file off"): `aesthetics_export`
  never removed a player or shared folder's own file on `DropFile` (only `settings.toml` and
  `portrait.*` by name) and the cascade followed a dropped link, so the drop did not happen;
  `validate/mod.rs` retains the kept files, `links.rs` skips a dropped link
  (`object_model.md` "What a scope drops", "Dropped link targets"). What follows: a shared
  boots folder left with no model is `shared_folder_no_model` and the player wears the
  game's own; a hidden collar FMDL is dropped and the kits keep their config's collar; a
  `.skl` beside a dropped FMDL gets no role and no finding (minor, Issues); a player's own
  all-hidden FMDL no longer shadows a linked shared folder's file of its name on PES 15-17
  (on Fox it still does, drawing nothing; rare, noted). (2) `prefox_face.rs` `insert`
  refuses a name the package holds in another case (`vtree::fold_name`); the CLI shape is
  reachable (`Face_High.fmdl` beside `face_high.mtl`: `folder_pack_failed`, where both
  `.mtl` files were packed before). (3) `tests/cli/main.rs` `Face` and `sorted_faces`: the
  hand split tests compare the multiset of faces (positions as `f32` bits, winding kept)
  of the source with the three packages', the Fox test swapping the `.model`'s winding
  back; both red with the right glove's bytes replaced by the left's. Tests: TC-MOD-69
  through the deep pass (`check` reports it too), the Common link and shared boots CLI
  tests, the deep unit test over the four kinds on PES 17 and PES 21, two ae tests, the
  `every_mesh_hidden` test, the `insert` unit and CLI tests. Files: `model_convert`
  `formats/fmdl/{mod,tests}.rs`; `aesthetics_export` `validate/{mod,links,tests}.rs`;
  `team_compiler` `deep/{mod,model,collar,portrait}.rs`, `processing/{conversion, mod,
  prefox_common, prefox_face, prefox_shared}.rs`, `tests/cli/{main, conversion,
  prefox_faces, prefox_hand_split}.rs`; object_model.md, messages.md, DECISIONS (lead).
  Gates green (`.tmp/4_y/gates_fix20_r2.log`, acceptance 311 of 313, TC-MOD-65/66 waiting on
  4.y-sub); `just bindings` 37 fixtures; mutants r1 37/28/8/1 missed (the link filter's
  position half, its test the rework), r2 37 mutants, 29 caught, 8 unviable, 0 missed
  (`.tmp/4_y/mutants_fix20_r2.log`).

- [x] 4.y-fix21 **The S18, S7 and S17 rounds' small fixes: TC-MOD-64 tells its meshes apart, a
  dropped shared link is no link, a textures-only folder is reported, a `.model` collar skips
  material losses** (rulings `.tmp/4_y/duck_rulings.md`
  "S18.A1", five of five accepted, so a second Astra round follows this rework; brief
  `.tmp/4_y/brief_fix21.md`, after 4.y-sub (b1) lands, never beside it). (1) moved to
  4.y-sub (b3), the maintainer's rule of 2026-10-10: a nested link names the same Common
  file as a root one, so the search resolves it, not excludes it, which also closes the panic. Was: `mtl_search.rs`
  `mtls_in` takes a `.mtl.common` link as a candidate wherever the model's folder is, while a
  link has a role directly in the player folder alone (`role_position`), so a nested link a
  lenient check keeps (`strict_file_type_check = false`, `parts/body.model` beside
  `parts/body.mtl.common`) is found by the search and its Common `.mtl` was never loaded:
  `files.get(..).expect` panics in `processing/model.rs` and `prefox_face.rs`. The search
  counts a link directly in the player folder alone, the nested one names nothing and the
  deep pass reports `model_material_undefined`. (2) TC-MOD-64's test clones the visible mesh
  as the hidden one, so dropping the wrong mesh passes: the hidden mesh gets its own
  geometry and the test asserts the survivor's is the visible mesh's. (3) `aesthetics_export`
  `validate/mod.rs` rebuilds a kept player's `links` from the unfiltered draft and `links.rs`'s
  orphan pass counts a dropped link, short of "a dropped link is no link" (DECISIONS
  2026-10-10): both leave a dropped link out; test: a consumer `DropFile` on `Crocs.boots`
  leaves the player's links empty and `Boots/Crocs` `shared_folder_orphaned`. S18.A1-2 (the
  singletons below a subfolder) is slice (b1)'s, checked at its review. From S7's second
  SWE-2 round and S17's first (rulings "S7.2", "S17.1"): (4) a folder that plans no
  package plans no textures task and each of its textures is `file_not_used` (decision;
  today `folder_tasks` plans the task with no package and `commit_folder` discards the
  converted textures with no finding); (5) `validation.rs` `model_name_messages`'s doc
  comment still cites "Reserved subfolders" and "in `face/`"; (6) `model_for_pre_fox`
  takes a `ConvertedMaterials` like its three siblings and the pre-Fox collar arm of
  `process_task` passes `StockCollar`, so a `.model` collar's conversion skips the material
  losses the plan's "Collars" paragraph says it skips; (7) `is_game_dummy`'s doc comment
  says the stem folds and the directory is exact. Deferred to the design-health pass, after
  (b2): `prefox_face.rs` at about 1,550 production lines (S17.1-3: the user-xml half as
  `processing/prefox_user_xml.rs`) and `deep/mod.rs` at 1,368.
  → verify: the three tests red first (the panic reproduced, the mesh swap, the dropped
  link counted); gates green; `just mutants-diff` over the rework.
  Done 2026-10-10 (sidekick, landed first time): (2) TC-MOD-64's hidden copy moved 1 on x,
  the survivor's faces asserted; (3) `links.rs` `standing_links` (the player's links less
  the dropped ones) feeds the cascade, the orphan pass and a kept player's `links`; (4)
  `roles.rs` `plans_a_package` (validation's reading of planning's rule, a unit test tying
  it to `plan_run` on five shapes and both engines) and `is_package_model` shared with
  `folder_tasks`, which plans no textures task without a package; (5), (7) comments; (6)
  `model_for_pre_fox` takes `ConvertedMaterials` and uses it only to choose the reported
  losses (a contradiction accepted: passing it to the writer renames a `.model` collar's
  materials, which the plan says it keeps). Files: `aesthetics_export`
  `validate/{links,mod,tests}.rs`; `team_compiler` `plan/{mod,roles}.rs`, `validation.rs`,
  `processing/{conversion,mod,prefox_common,prefox_face,prefox_shared}.rs`, `tests/cli/
  {conversion,prefox_ingame_face,referees}.rs`; `model_convert` `formats/fmdl/mod.rs`.
  Mutants-diff `fbed127`: 30 mutants over both halves, 25 caught, 5 unviable, 0 missed.
  Its report's three observations (a no-package folder's texture link unreported, and two
  shapes to check) are 4.y-fix22 (12).

- [x] 4.y-fix22 **The S13, S15 rounds' rework: a left-out kit number's file stays out, a
  Common `.mtl`'s stems resolve in Common, a glTF-dropped folder's `fpc_on` does not count**
  (rulings `.tmp/4_y/duck_rulings.md` "S13.2", "S15.1a", "S15.1b"; after 4.y-fix21,
  which owns `plan/mod.rs` and `validation.rs`; items (9)-(11) from S18's first SWE-2
  round, "S18.1").
  (1) `kit_variants.rs` `model_variant_sets` dedups a second file of a left-out kit
  number as another part of that kit, so it merges into every kit's model: only the used
  number's files dedup, a left-out number's are left out and named (S13.2-1). (2)
  `file_role_messages` and `has_effective_part` read variant sets from the folder's own
  files, compile from the part's sources: both read `part_source_models` (S13.2-2). (3) a
  folder `.model` converted with a `Common/` `.mtl` looks in the folder first
  (`PartTextures::CommonSet`), against "a stem resolves in the folder of the material
  file that set it": it points at Common's textures alone (S13.2-3). (4)
  `texture_supply` takes one set (S13.2-4). (5) `EffectiveTeamKitFpc::of` reads the
  roster-mapped folders, so a glTF-dropped folder's `fpc_on` does not count (S15.1a-1).
  (6) a unit test whose export order is not name order (S15.1a-2). (7) `keeps_archive`
  private (S15.1a-3). (8) `overrides::list`: only a missing path means no overrides,
  anything else unlistable is the error, with tests of both error arms (S15.1b-1, -3).
  (9) `tests/cli/conversion.rs`'s "reserved subfolder" comment (S18.1-C1). (10)
  `PreFoxPart::xml_type`'s doc names `face_xml::xml_type` (S18.1-C2). (11) a dropped
  link's `model_hidden_dropped` merged into the player pass's findings at the link's
  position, as the module doc's "file order" says (S18.1-C5). S18.1-C3 (a shared
  face's nested file a member's xml packs) is slice (b2)'s, its brief carrying the test.
  (12) a no-package folder's texture link (`skin.dds.common`) is `file_not_used` too, and
  two shapes 4.y-fix21's report named are checked (4.y-fix21's report, O1-O3).
  → verify: tests red first for (1), (2), (3), (5), (8); gates green; `just
  mutants-diff` over the rework; S13's third SWE-2 round on its diff.
  Done 2026-10-10 (sidekick, landed first time): all twelve as briefed; `combined_folders`
  moved to `plan/roles.rs` taking `hand_weighted`, one copy for planning, the deep pass
  and validation; a shared folder's roles read from its own files, as the deep pass
  does; TC-MOD-54's last case updated to the ruled rule (and its plan text with
  `pipeline.md`'s, which still said folder first). Files: `team_compiler`
  `deep/mod.rs`, `kit_variants.rs`, `plan/{mod,overrides,roles}.rs`,
  `processing/model.rs`, `validation.rs`, `tests/cli/{common_links,conversion,models,
  prefox_faces,referees}.rs`. The `#[cfg(unix)]` non-UTF-8 test runs first in CI.
  Residuals, step 4.y-fix23.

- [x] 4.y-fix23 **4.y-fix22's residuals** (its report, `.tmp/4_y/sk_fix22_report.md`).
  (1) A PES 17 referee whose only model is a `.model.common` link gets no face
  (`is_package_model` does not count the link), while the converted Common model is still
  written into the refs CPK with nothing naming it: the link must count as his face part,
  as a team player's does. (2) A Fox referee planning no package who links a face folder
  holding textures alone: compile reports `link_combined` and writes nothing, with no
  finding on the textures. (3) The deep pass computes the kit-FPC status before planning
  drops glTF folders, so `check` still counts a glTF-dropped folder's `fpc_on` where
  `compile` does not (4.y-fix22 (5)). (4) A combined shared face's own left-out variant
  still gets `fmdl_fcl_hair_fallback` on the shared folder, read alone (one player linking
  it leaves it out, another may not): by design, noted. S13's third SWE-2 round (rulings
  S13.3, all three accepted; S13's loop ends): (5) a per-kit set spanning two combined
  shared folders, or in a pre-Fox `ingame_face` player's linked face, is left out with no
  `kit_variant_model_left_out` (`kit_variant_model_messages`'s `holds_own`): a player
  reports such a set unless every file of it sits in one shared folder that reports its
  own sets; (6) on Fox a shared `Boots/` or `Gloves/` folder no linking player combines,
  holding a model of another package's name (`glove_l.fmdl`, `face_high.fmdl`,
  `pants.fmdl`), reports `file_not_used` for it, not silence or `fmdl_fcl_hair_fallback`;
  (7) `kit_variant_model_messages` reads `combined_folders`.
  → verify: (1)-(3), (5), (6) tests red first; gates green; `just mutants-diff`.
  Ruled 2026-10-11 (the lead, from a code map at `e3ce7e0`, `.tmp/4_y/fix23_map.md`;
  DECISIONS 2026-10-11, two entries):
  - (1) deferred until the cup next plays a pre-Fox version (DECISIONS 2026-10-11), an
    open issue.
  - (2), (3) and (5) closed without a change (plan `pipeline.md` "not worth its rule" for (3)).
  - (4) by design, as noted.
  - (6) holds today only through a shared folder's `.common` model links (a model file of
    another package's name there is already `fmdl_name_invalid`): the name rule extended to
    those links on PES 18-21 (`player_folders.md`), not the S13.3-2 `file_not_used`.
  - (7) the refactor, behavior unchanged.
  Briefed: (6) and (7), `.tmp/4_y/brief_fix23.md`. Done 2026-10-11 (SWE-2, one run): a
  shared `Boots/` or `Gloves/` folder's model link is named by the suffix rule on PES 18-21
  (TC-MOD-83); `kit_variant_model_messages` reads `combined_folders`. Files:
  `aesthetics_export` `validate/{folders,tests}.rs`; `team_compiler` `plan/mod.rs`,
  `tests/cli/common_links.rs`.

- [x] 4.y-mattable **The material table checked against the community's shader reference**
  (the maintainer, 2026-10-10). `resources/Materials.wikitext` (the cup wiki's "Materials"
  page: pre-Fox `.mtl` settings and the `Basic_C`, `Basic_CNSR` and `Shadeless` shaders; Fox
  material settings, texture paths and the Generic, Metallic, Fresnel, Constant, Glass and
  UV-scroll/step shaders, each with its shader, technique, parameters and texture roles) is
  compared, row by row, with `model_format.md` "Shader families", "Textures", the
  `[mat.fox]` and `[mat.prefox]` tables, and with what `model_convert` writes
  (`materials/{family, to_fox, to_prefox}.rs`): each family's shader and technique names,
  its texture roles and their color spaces (`_SRGB`/`_LIN`/`_NRM`), the parameters and
  their default values (`MatParamIndex_0`, `Reflection`, `Shininess`, `Incidence_*`), and the
  required defaults the wiki names (Metallic's `SpecularMap_Tex_LIN` at `#FF6C3A`,
  `MetalnessMap_Tex_LIN` at `#FFFFFF`). A shader the wiki has and the table lacks is listed,
  not added: whether a family is worth having is the plan's call. Each mismatch is ruled:
  the plan or the code wrong (a step of its own, briefed), or the wiki (noted, with the
  evidence, a game file or an in-game check). → verify: a table of every row compared and
  its outcome in `.tmp/4_y/mattable/`; each mismatch a ruling, and a step where it is ours.
  Done 2026-10-11: 143 rows (`table.md`, 80 match), every mismatch ruled (`rulings.md`)
  against a census of every FMDL and `.mtl` on the machine (7,315 and 10,101 read). No code
  change: the tables stand where real files use our form; the three differences real files
  favour the wiki on are family defaults only glTF reaches on Fox, settled before Phase 7
  (`model_format.md` "Checked against the community's reference", `development_plan.md`
  "Phase 7"); the Textures table's dummy fallback names its families; one pre-Fox open issue
  (spaces in a converted `.mtl` path). DECISIONS 2026-10-11.

- [x] 4.y-simp **The converge's simplification pass** (4.y-conv's design-health item; the
  maintainer, 2026-10-11). Ruled from a survey of `team_compiler`, `aesthetics_export` and
  `pipeline` (DECISIONS 2026-10-11 "the converge's simplification pass"):
  - a `shared_texture_conflict` fails the player's textures task, the package-level salvage
    removed (TC-TEX-18 rewritten);
  - a Common texture conversion alone refuses fails the Common textures task;
  - an undecided root returns the parse's findings alone;
  - `multiple_ref_exports` loses its Run-scoped summary;
  - no-behavior cuts: `FolderModels::of_player`, one file-naming helper, `check_portraits`'
    recount, `standing_links`, the other-engine model-format match, the canonical-order
    winner, a stale `PlanReport` doc line.
  Rejected, with reasons: `.tmp/4_y/simp_rulings.md`. Briefed: `.tmp/4_y/brief_simp.md`.
  → verify: gates green; `just mutants-diff`; acceptance unchanged in count.
  Done 2026-10-11 (SWE-2, two reworks): net about -270 lines in `team_compiler` and
  `aesthetics_export`; the conflict's `packages` names every package claiming the path;
  `processing::Finding` lost its always-`Keep` disposition; TC-TEX-12 widened to a Common
  texture only conversion refuses (BC6H), with its CLI test. Astra: 5 then 3 concerns, all
  accepted (`.tmp/4_y/simp_rulings_a1.md`); the stale run-summary text left
  `refs_arranger.md`, `core/parallelism.md` and `core/architecture.md`. Gates green (2,780
  tests); `mutants-diff 84a55b5` every viable mutant caught on each round.
- [x] 4.y-scen **Every CLI test cites a scenario** (the maintainer, 2026-10-10;
  `CONTRIBUTING.md` "Acceptance IDs", DECISIONS "every CLI test of a tool cites a
  scenario"). `scripts/acceptance.py` gains the check, by the lead (the scanner is
  correctness-critical), with its case in `scripts/acceptance_test.py`: every `#[test]` in a
  tool's `tests/cli/` carries an ID comment directly above it, listed by `report`, failing
  `strict`. Then the uncited `team_compiler` CLI tests (235 of 526 after 4.y-sub (b3b1), counted
  by `.tmp/lead/scenario_coverage.py`) are triaged: each cites the scenario it proves, or an
  existing scenario is widened where the test is a variant of it, or a new scenario states
  the behavior; written by the lead in `team_compiler/README.md`'s Acceptance section, the
  citations added by a briefed sidekick. After 4.y-sub, whose slices add CLI tests with their
  scenarios. The new and widened scenarios get the cross-family critique `AGENTS.md` asks for
  after Acceptance text. → verify: `just acceptance strict` lists no uncited CLI test; the
  scenario count before and after; the critique's rulings.
  Done 2026-10-11: the scanner landed in `84a55b5` (241 uncited at that HEAD). Triage by three
  read-only agents, ruled by the lead (`.tmp/4_y/scen/rulings.md`): 20 new scenarios
  (TC-SRC-10, TC-CLI-10 to 12, TC-MOD-84 to 90, TC-TEX-19 and 20, TC-XML-16 and 17, TC-CMN-32
  to 35, TC-DEP-15 and 16; TC-TEX-19 split in review), about 60 widened, one rule each;
  citations added by a checked script (236), four redundant tests deleted, one moved to a unit
  test (`lib.rs`), the two TC-MOD-84 tests given output assertions. Scenarios 342 to 363, all
  proven; `acceptance strict` lists no uncited test. Astra (checkpoint a): six rounds, 21, 11,
  6, 9, 6 and 3 concerns, all accepted, every one a scenario text claiming more or other than
  its tests prove; no code behavior changed.
- [~] 4.y-conv **Converge** (`AGENTS.md` "Closing a phase" (1)): the lead's audit of
  `team_compiler`, `aesthetics_export`, `pipeline` and the Phase 4 edits of the lib crates
  against `development_plan.md` "Phase 4", the `pipeline.md` walkthrough, `messages.md`,
  `settings.md` and every TC ID; `just acceptance strict`; the design-health pass (design-tell
  sweep, `just mutants` per crate with every survivor triaged, `just clef` per crate with every
  flag ruled, `pub` census, and a simplification pass: low-hanging cuts only, each a special
  case a general rule already covers, or machinery whose only gain is salvaging more of an
  export that already reports an error, which a person looks at anyway; the cuts briefed as one
  slice, no redesign); the census over
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
  questions moved to "Resolved decisions"; the worklog's step list collapsed to the phase row;
  the dated log moved out of the worklog into its own append-only file (`docs/LOG.md`, a row in
  `AGENTS.md`'s document table: the worklog holds where we are, the log what happened; the
  maintainer, 2026-10-11, the worklog having grown past 7,000 lines) →
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
with them on the socks; the shorts keep their layout across engines, so the table covers the
socks alone (DECISIONS 2026-10-04; the maintainer confirmed it 2026-10-10: "time will tell";
`pipeline.md` "Layout conversion").

### Phase 5 — Savefile integration

Steps are itemized when Phase 4 closes; these are fixed already:

- [ ] 5.0 Check the latest commits made to the unofficial 4ccEditor fork (`Tools_4cc/4ccEditor-1`,
  origin `AnonymousClouds/4ccEditor`; `pes_savefile/README.md` names it the reference): it is
  under constant development. Fetch, then read every commit after `4a95b7c` (2026-06-19, "Add
  Tactics tab for 16-21. Add ability to import Texports from 15-21"), the local clone's head at
  Phase 2's close → done when: each savefile-relevant
  change (schema fields, write behavior, `.4ccs`/`.4cct`/Texport formats) is reflected in
  `pes_savefile` and its plans or recorded as not applying, before any Phase 5 writing code, and
  the commit reached is in the log
- [ ] 5.0b Prototype the two mutation kinds `cargo-mutants` lacks (after Phase 4's converge, so
  it competes with no review): (1) drop a guard, an `if` whose body returns, `continue`s,
  `break`s or errs, its condition replaced with `false`; (2) remove one element of a literal
  list a `split`, `contains` or `matches!` decides by. S10's review (2026-10-09) found the
  color grammar's leading-`+` guard and tab refusal untested with every mutant of the file
  caught (`CONTRIBUTING.md` "Mutation runs"). Upstream checked 2026-10-09: issue #180 "Delete
  statements" open since 2023 with no milestone, its notes leaning against deleting statements
  that hold a `return`; nothing on literals. Shape: a `syn`-based generator writing one patch
  per mutant, run through `scripts/mutants.py`'s tree copy, timeouts and remote split.
  **Needs the maintainer's yes first**: a new dev-tool crate (linked into no release binary)
  and a direct `syn` dependency (agreed in principle 2026-10-09). `syn` 3 with `full` and
  `visit`, features the workspace already builds it with, so it compiles nothing new (check:
  a clean build compiles `syn` 3 once): the generator only reads the tree and patches text,
  so no `visit-mut`; not `syn` 2, which leaves the tree as `binrw` and `pyo3` move on. → done when: run over `aesthetics_export`, its survivors
  triaged into real gaps and equivalents, and the count decides whether it joins converge
  (worth it when at least about half the survivors are real gaps)
- [ ] 5.0c Red behavior inventory (maintainer, 2026-10-09; after Phase 4 closes, before Phase 5
  builds on the compiler): go through Red's own code (`4cc-aet-compiler-red/Engines/python/`,
  not its bundled site-packages) module by module, one subagent per module, and list every
  observable effect it has on what it is given: each file it writes, renames, rewrites, skips
  or refuses, and each value it changes. Map each row to the plan section or TC ID that covers
  it, or to a decision entry recording a deliberate deviation; an unmapped row is a gap and
  becomes a step. An inventory, not a line-by-line diff: Red is evidence of *what* to produce,
  not of how the code is organized (`AGENTS.md` "Fundamental concepts"), and the parity tests
  (`tests/parity_prefox.rs`, the Fox parity rows) cover only what their fixtures exercise. It
  finds what Red does that the plan forgot; it cannot find defects in what Studio adds on top
  (the export layout, links, kit tokens, conversion, the deep checks), which the reviews cover
  → done when: every Red module's rows are mapped or turned into steps, the table kept under
  `.tmp/5_0c/`, and its gap count in the log

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

- open (2026-10-11, 4.y-mattable row 27; deferred until the cup next plays a pre-Fox version) —
  the cup wiki says a `.mtl` splits a path at spaces, and 212 of 60,027 texture names in real
  FMDLs hold one (`Kit 1_u0XXXp0.dds`), so a Fox model converted for PES 15-17 may write a
  `.mtl` path the game cuts short. Unverified in game.
- open (2026-10-11, 4.y-fix23 (1); deferred until the cup next plays a pre-Fox version) — a
  PES 17 referee whose only model is a `.model.common` link gets no face (`roles.rs`
  `is_package_model` does not count `PreFoxCommonModel`), while the converted Common model is
  still written into the refs CPK with nothing naming it; the link should count as his face
  part, as a team player's does.
- resolved (2026-10-11, by 4.y-simp: the conflict leaves the player folder out; B3C2A.A2-1) — a `shared_texture_conflict` drops the losing package's
  task, and with it every texture only that folder held: a surviving model of another
  combined folder naming one of them (a face model naming a texture of the shared boots
  it combines) then names a texture the CPK lacks, with no finding. Two differing copies
  did so before shared folders' links; no export has shown the shape. Revisit with the
  design-health pass.
- open (2026-10-11, S19.1-B3) — which material files a glTF in a player or shared folder's
  subfolder reads (`model_format.md` "Name matching" is written for one flat folder; the
  `.mtl` search's two folders, the file's then the model folder's, are the likely answer):
  for Phase 7's plan pass, when glTF sources are planned.
- resolved (2026-10-10, 4.y-fix20) — a Fox model whose every mesh is hidden, beyond the
  files the scan found (4.y-fix13): a boots folder whose every part is hidden failed its
  task with "no parts to merge", a gloves folder whose every side is hidden wrote a
  `glove.xml` with no entries, and a player's `.fmdl.common` link to an all-hidden Common
  FMDL left his face xml naming a file the Common task never wrote. The deep pass now drops
  such a file before any task is planned, so a shared folder left with no model is
  `shared_folder_no_model` and a link is dropped with its target. Left minor: a `.skl`
  beside a dropped FMDL gets no role and no finding (the sidekick's probe); and a player's
  own all-hidden FMDL no longer shadows a linked shared face's file of its name on PES
  15-17, where on Fox it still does (nothing drawn either way for his own part).
- open (2026-10-09) — a Fox referee's body under `fpc_off` waits on a Fox refkit, an FMDL
  body the maintainer will make ("We'll make one later"; `blue_port.md` "The referee
  body"): then the Fox referee template ships it and a step merges it into an `fpc_off`
  referee's face package. Until then `fpc_off` applies its settings preset alone on Fox.
- open (2026-10-09) — per-kit face packages on Fox wait on FoxDen (`model_format.md`
  "Kit-dependent assets", DECISIONS 2026-10-09): each kit number gets a face package of its
  own, merged from that number's variants of any file kind, once FoxDen loads one per kit
  picked; the layout it reads is decided with FoxDen's work, then a step in the phase open
  at the time builds it and retires `kit_variant_model_fox`. FoxDen is a work in progress.
- resolved (2026-10-10, 4.y-fix16) — on PES 15-17 the deep pass reads a player folder's models
  with the roles of a folder without `ingame_face` (`FolderModels::of`), so under the marker
  a per-kit model variant that planning leaves out is still read and checked: a broken
  `boots_kit2.model` beside a valid `boots_kit1.model` drops a marked folder `compile`
  would build. The pass now reads the roles as planning does (`part_source_models`).
- open (4.y-fix17's review, 2026-10-10) — a PES 17 `.model` collar compiled for PES 15 or 16
  keeps its `uni_collar` material, packed as is when the pre-check does not flag it (PES 16)
  or moved onto the skeleton when it does (PES 15), while those versions' `uniform.mtl`
  defines no `uni_collar` (the FMDL conversion names `uni_shirts` alone for them, DECISIONS
  2026-10-09). What the game draws for a collar material its `uniform.mtl` lacks is unknown:
  an in-game check on PES 16 (the F: install), and if it draws nothing, the same-engine
  collar gets the version's names as the converted one does.
- resolved 2026-10-10 (4.y-fix17) — a texture whose bytes open with no signature the
  deep pass knows (four junk bytes as `Common/boots.dds`) gets no finding there (the texture
  check reads headers, `texture_finding`), reaches the Common textures task and fails it
  with `folder_pack_failed` at `Common` (`DropFolder`: every Common texture lost, the
  players keeping theirs). The rule says the pass checks what `compile` reads; a texture
  the decoder refuses should be its finding (`texture_codec_unsupported` or a header code
  of its own, the file alone), on a folder's texture too. For the next rework brief.
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
- closed 2026-10-10 (by the subfolder rule, step 4.y-sub) — two reserved subfolders holding
  one texture under one stem with identical bytes (`boots/outline.dds` and
  `gloves/outline.dds`, the maintainer's `refs fox test` and `refs lan party` exports) were
  `texture_stem_conflict`. Under the rule each subfolder is a namespace of its own, so the
  pair is no conflict; those two exports are AET referee layouts, refused with
  `player_layout_proto` until upgraded.
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
  version's encoding clamps, as `kit_config_version_clamped`, step 4.9a). The maintainer's
  rulings (2026-10-10): the three "requires model 144 or 160" warnings are reported as one
  Warning, `kit_config_option_ignored` (step 4.y-kitwarn); the two Infos
  (`kit_shirt_model_unknown`, `kit_unknown_sleeve_value`) stay unreported (step 4.y-ingame2
  (e): the game draws both values, no crash; DECISIONS 2026-10-10);
  `kit_collar_zero` (the lib's one Error; `messages.md` says `kit_config_invalid` covers a
  config that "fails to validate", which the deep pass does not do) waits for a collar
  census: before a kit is dropped for a zero collar, count how many real configs carry one
  (the AET exports' `Kit Configs/` and the installs' CPKs), eventually.

- open (4.y-fix11's review, 2026-10-10) — read in the Fox-to-pre-Fox direction,
  `KIT_LAYOUT_REMAP` puts two right-sock stripes (pre-Fox 2008-2040 and 1656-1688)
  exactly 6.00 units off their range's centre, TC-KIT-19's tolerance: the test passes
  at its edge, and one texel more would fail it. Nothing in game shows it (the
  maintainer's PES 21 view of design A sits as drawn, 4.y-ingame2). Check the right
  sock's column of the table against the models at the design-health pass before
  widening the tolerance.

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
  residency the budget forbids (`libs/pipeline.md` "What a solid `.7z` is charged"). The
  maintainer's answer (2026-10-10): a small archive is kept, step 4.y-7z, done the same
  day: the `.7z` half of this issue is closed; the folder half (the file cache's second
  read) stands.
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

The dated log is `docs/LOG.md`.
