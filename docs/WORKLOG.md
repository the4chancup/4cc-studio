# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 8 (GUI) is next in the phase order, after the DBC work below. Phase 6 (Export
upgrader) closed 2026-10-20; Phase 5 (Savefile integration) 2026-10-17. Phases 1 to 4 done
(Phase 4 closed 2026-10-12, Phase 3 2026-10-02, Phase 2 2026-09-30). Built ahead of their
phases: 10.0a-e (`music_export`, its plan section, `pub` audit and whole-crate mutation run
done; converging with Phase 10), 16.w (the platform floor) and 20.0a-d (the Ruleset editor's
coverage table and draft, the DBC7 ruleset).
**Next:** for Divegrass Before Christmas (exports due 2026-11-18, the Save and Ruleset editors'
first test ground; `plans/core/development_plan.md` "Divegrass Before Christmas"): Phase 8
(its steps itemized: the shell, 8.6a-c and 8.6d1 done, 8.6d2 the help window next; then the Save
editor's view), Phase 20 (the Ruleset
editor), the 0.0.1 preview build for DBC (the two editors; the Team compiler and Export
upgrader greyed out), then Phase 8's other views. Done for DBC so far (2026-10-20): 20.0d (the
DBC7 ruleset), 8.4 (the usability pass; `PROPOSALS.md` P1-P22 wait for the maintainer's
rulings) and 8.5 (the Save editor's draft, `docs/prototypes/save_editor.html`, waiting for the
maintainer's remarks; the view starts with what they are least likely to change). The face_diff model comparison (Issues, "a face diff is
engine-specific") follows the editors. 8.0's acceptance sections are reviewed and the shell's
draft (`docs/prototypes/shell.html`, step 8.2) waits for the maintainer's remarks.
Side-leads are
paused (single lead + SWE-2); the record is `.tmp/lead/tracks.md`. 2.5b (GPU BC7) is step 16.x, its `dds_convert` half built ahead of its
phase (16.x a, unwired)
(decision entries 2026-09-21 and 2026-09-28). Release target (2026-09-28): 0.1.0 after Phase
8; phase order 1–6, 8, 0.1.0, 7, 9–16 (`plans/core/development_plan.md` "Releases");
first-class target the Fox version the cup moves to around April 2027 ("Target versions").
**Blocked on:** nothing.

**Working mode (since 2026-10-10).** The lead (Claude, in Claude Code) plans, briefs, reviews
and writes what is correctness-critical; the sidekick, SWE-2 through the Devin CLI, implements;
the cross-family reviews run through the `duck` skill (Astra), the lead running them itself.
When a review stops for quota, it is queued here (commit range, plan sections) and retried
once an hour; work continues past it, but no phase closes with one outstanding.
**Handover queue:** empty; every review Phase 5 queued has run (rulings under `.tmp/5_*/`).

---

## How to resume

| Thing | Where |
|---|---|
| Plan index | `docs/plans/README.md` → `docs/plans/core/README.md` |
| Development phases | `docs/plans/core/development_plan.md` "Development Plan" |
| Legacy tools (format evidence) | `docs/spec/core/README.md` "Project context" (paths are per-machine) |
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
- `dds_convert`'s CPU BC1 core (`block_compression`, PCA plus one refinement) trails DirectXTex
  by about 2.5 mean error on an 8x4 grey-ramp mip while matching it within 1.0 at the top mip;
  the encode tests compare the chain-wide mean for that reason. Revisit with representative kit
  and face textures when the Team compiler can produce them (plan: "tune against representative
  textures").
- `just mutants-diff` reuses `mutants.out/`: it wipes a whole-crate run's results. Keep the
  whole-crate run's summary aside (to `.tmp/mutants_<step>_whole/`, only `*.txt` and
  `outcomes.json`: `RUNBOOK.md` "The scratch folder") before the first diff run.
- `block_compression` 0.10's BC3/BC4/BC5 decoder truncates the alpha-ramp interpolation where
  DirectX rounds; `dds_convert::dds::fix_interpolated_channels` re-derives those channels. Drop
  it if a later release rounds (the exact-match test will say).
- While a PES game (or any process named `PES2015.exe` to `PES2021.exe`) runs on the machine,
  `save_editor`'s tests of `import-toml`, `apply-patch` and `transplant` on that version's
  fixture save fail with "PES 20xx is running": the commands refuse then (SE-CLI-02). Close the game before `just gates` or a
  mutation run that touches `save_editor`; a failed unmutated baseline is the symptom.
- `studio_core`'s `pes_process.rs` is split by `cfg` per OS, so a mutation run reports the other
  OS's mutants as missed: a Windows-only function's mutant tested on the VPS's Linux half (and a
  Linux one on this machine) compiles out. Triage those by OS, not as test gaps. Windows gates
  never compile a `target_os = "linux"` branch either (5.4a6's did not build): `cfg`-split code
  gets `just gates-remote` before it lands.

---

## Phases

Status: `todo` · `in progress` · `done` · `blocked`. A phase is done only after its last two
steps, which every phase has: **converge** (the lead's own audit first, then the cross-family
reviewer's, both against the phase's plan sections and acceptance IDs; each gap becomes a new
step above it, and the phase waits for them) and **rewrite**
(what the phase delivered moved from `docs/plans/` to `docs/spec/`, corrected where it was built
differently; `AGENTS.md` "Working documents"). Then collapse its step list
below to this one row, keeping its `manual: checked` lines; the step-level detail stays in git history. Tool phases (3–6, 8–15, 17, 19, 20) also
open with an **acceptance** step: the tool plan's "Acceptance" section for that phase, written
before any code (GUI scenarios that no automated test can prove are marked `manual` and proven by
a recorded check at converge — `CONTRIBUTING.md` "Testing"). Procedure: `AGENTS.md` "Working
documents".

| Phase | Scope | Crates | Status |
|---|---|---|---|
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree`, `pes_version` | done |
| 2 | Library crates (standalone-verifiable) | `wezlib` `cpk` `fpk` `ftex` `dds_convert` `fmdl` `pes_model` `uniparam` `fox2` `archives` `fpc` `teams_list` `kit_config` `color_tools` `elevation` `model_convert` (native) `pes_savefile` `python_bindings` | done |
| 3 | Team compiler skeleton | `team_compiler`, `aesthetics_export`, `pipeline` | done |
| 4 | Processing logic | `team_compiler` (`plan/` `processing/` `bins/` `output/`), `aesthetics_export` deep validation | done |
| 5 | Savefile integration | `save_editor` logic, `aatf`, `team_compiler` `output/savefile.rs` | done |
| 6 | Export upgrader | `export_upgrader` | done |
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

Done 2026-09-13 (spec now describes what exists: `docs/spec/core/development_plan.md` "Phase 1"). Step detail
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
now describes what exists: `docs/spec/core/development_plan.md` "Phase 3",
`team_compiler/README.md` ("Acceptance": 77 scenarios, 75 cited by tests, 2 manual),
`spec/team_compiler/pipeline.md` "Run driver shapes", `aesthetics_export/object_model.md`,
`libs/pipeline.md`. Step detail (3.1-3.9f, the 3.w `.cargo/mutants.toml` audit, the 3.y
converge with its whole-crate mutation runs, the 3.z shell slice, 3.x this rewrite) in git
history up to commit `32261dd`. Carried forward: the phase's open issues under "Issues".

TC-GUI-01 manual: checked 2026-10-02 by the maintainer on Windows, the window, the sidebar's
Team compiler entry, its Settings, Compile button and log.
TC-GUI-02 was checked by hand the same day (the /egg/ tracer export compiled from the window,
the log showing `egg Tracer: Info export_identified (team=/egg/, id=792)` then `Run finished:
exit code 0`); since 8.0 it and TC-GUI-03 are automated, cited by the headless view and shell
tests, and TC-GUI-01 is narrowed to the window opening. Linux: not yet checked, no machine
with a display.

### Phase 4 — Processing logic

Done 2026-10-12. Spec now describes what exists: `docs/spec/core/development_plan.md` "Phase 4",
`team_compiler/README.md` ("Phase 4 scope"; "Acceptance": 363 scenarios, 2 manual, 5
withdrawn, all proven, every CLI test citing one), `team_compiler/pipeline.md` (the walkthrough
and "Run driver shapes"), `messages.md`, `settings.md`, `aesthetics_export/object_model.md`.
Step detail (4.0-4.34; the 4.c Clef passes; the 4.y converge: the lead's audit and the export
census, 4.y-fix1 to 4.y-fix23, 4.y-sub, 4.y-7z, 4.y-kitwarn, 4.y-mattable, 4.y-simp,
4.y-scen, the S7-S20 review rounds, the in-game harness and checks; 4.z this rewrite) in git
history up to commit `c93f025`. The converge's closing whole-crate mutation runs, all on bonfire
over `6e13fae`: `aesthetics_export` 519 caught, 0 missed; `pipeline` 54 caught, 6 missed
(`available_memory`'s arms for other platforms, accepted in `.cargo/mutants.toml`), 8
timeouts (`MemoryBudget::acquire` waits a mutant makes endless, caught); `team_compiler`
2205 caught, 10 missed, 3 timeouts, 384 unviable: the five sites filing a texture as the folder's own or a combined folder's got tests (`c93f025`), `own_package`'s
`ingame_face` guard is excluded as read by no consumer, the three `deploy.rs` error-kind
guards (tests Windows-only; the run was Linux) and the timeouts (two endless loops, one
64-fold texture) were caught when rerun on Windows, `settings_view` is the placeholder
label. Carried forward: the Fox cube map check (4.y-ingame2 (d), waiting on a hand-made FMDL; its
`QUESTIONS.md` entry), the phase's open issues under "Issues" (the pre-Fox
ones deferred until the cup next plays a pre-Fox version), the in-game questions in
`docs/QUESTIONS.md`.

The phase's in-game checks (TC-REF-04's at 4.19d; 4.y-ingame; 4.y-ingame2):

4.19d:
manual: checked 2026-10-09 by the lead on PES 17 through the harness (`.tmp/4_19/ingame/`, results in `.tmp/4_0/apptest/results.txt`): the referee present with the pair installed, nothing drawn; a `.mtl` beside a nocloth `.model` is read and stops the model drawing (runs E and F against 2026-10-08's run I), so the pre-Fox marker goes by Red's route instead: step 4.19f, DECISIONS 2026-10-09; the scene where the pre-Fox marker shows is a maintainer question

4.y-ingame:
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

4.y-ingame2:
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
manual: checked 2026-10-12, the first PES 20 `.ted` (g): team /3/ exported from
`F:/Games/PES2020` (the cup's base DLC; `game.py`'s new `20` entry, no Sider; the file
name typed by posting `WM_CHAR` to the window, so no focus taken; frames
`.tmp/4_y/ingame/frames/p20/`) is 14760 bytes (`0x39A8`, the derived size) with key
index `0x14`, the only one that decodes the coach block's `MANAGER` and the team id.
The writer's PES 20 header template was PES 21's, whose u32 at `0x08` (the body length)
is 21's: replaced by the file's header, and the file is the PES 20 fixture of
`pes_savefile`'s Texport tests (`pes20_texport.ted`). Its 660-byte block holds five
125-byte runs where the other fixtures' is zero (`operations.md` Texport). (g) closed.
manual: checked 2026-10-12, the lower sock on PES 21 (a's rest): Test 3's kit (design A,
`pre-fox`, converted) compiled for `/3/` (701, `build_socks_export.py socks team=3
runner=21 noplayers`, the release build of `5a28e37`), whose second row is a standard
body in the game's boots: the green diagonal band on the lower calf crossing the white
stripe, each leg's in the PES 17 reference's direction, and the orange hoop at the
ankle, above the boot (`.tmp/4_y/ingame/frames/p21sock/s06_front.png`,
`s07_side.png`, `s08_back.png`, against `s17_34_out_legs.png`); with the upper part
checked 2026-10-10, the converted sock matches PES 17 feature for feature. (a) closed.
manual: checked 2026-10-12, QUESTIONS "Non-power-of-two mipped textures and kits over 2048
on Fox": Red (`e12aa01`, a scratch copy under `.tmp/5_0e/red_np2/`, `setup.py` and
`retarget.py`) compiled EGG VGL26's `Faces/XXX14` and AKG VGL26's kits as `/3/` (the
save's /vg/ teams are placeholders that draw no custom kit, `s06_vgl5.png`); its FTEX keep
the chains (`screen` 512x461 and `text` 797x64 ten levels, `u0701p1` 4096 thirteen). In
Edit mode row 14 (70114) draws the Steam page and the caption as the sources show
(`.tmp/4_y/ingame/frames/p21np2/s09_row14_face.png` against `screen_source.png`), and row
2 wears AKG's kit (`s07_kit4096.png`). Red's rules adopted (DECISIONS, step 5.0f), the
entry removed. The gloves-subfolder entry (PES 17) cannot run yet: a shared gloves folder
compiles to the team's first shared ID, drawn only on a player the save points at it, which
is the deferred pre-Fox patch's write; the entry says so.

### Phase 5 — Savefile integration

Done 2026-10-17. Spec now describes what exists: `docs/spec/core/development_plan.md` "Phase 5",
`pes_savefile/README.md` (and `operations.md`, `model.md`), `aatf_rules.md`, `save_editor.md`
(its CLI and 25-code message catalog), `team_compiler/README.md` ("Acceptance": the SE and TC
scenarios, 442 proven under `just acceptance strict`), `aesthetics_export/settings_toml.md`.
Step detail (5.0-5.0m4: the 4ccEditor fork survey, the Red inventory and its Fox gaps, the
plans/spec split; 5.1a-d the schema widening to PES 18-19; 5.2a-i `aatf`: schema, interpreter,
host, the eight rulesets, their review loops; 5.3a-b the Team compiler's rows, patch and
savefile transaction; 5.4a-c the Save editor's CLI; 5.5 the in-game check; 5.y-gloves; the
5.6a-f converge: the lead's audit, the design-health pass, the `pes_savefile` census, the
cross-family reviews, the citation audit, 5.6e this rewrite) in git history up to commit
`27a94bb`. The converge's closing whole-crate mutation runs (extra kinds included, split with
bonfire, triage in `.tmp/5_6/close/mutants_<crate>/triage.md`): `pes_savefile` 1830 caught, 2
missed (both tested, `27a94bb`), 2 timeouts (endless loops, caught); `aatf` 242 caught, 0 missed, 1 timeout (an engine with no operation limit hangs the endless-loop
test, caught);
`save_editor` 128 caught, 3 missed (the placeholder `view` and `settings_view`, and a
`NotADirectory` arm covered on Linux); `studio_core` 76 caught, 5 missed (`pes_process.rs` arms
the half's OS compiles out); `aesthetics_export` 603 caught, 0 missed; `team_compiler` 2474
caught, 1 missed (the placeholder `settings_view`), 3 timeouts (caught). Carried forward: the
gloves half of the in-game check (issue "FoxDen's PES 2021 gloves trampoline"), the pre-Fox
patch and the Red inventory's pre-Fox gaps (deferred until the cup next plays a pre-Fox
version), the placeholder views (issue of 2026-10-16), the phase's other open issues under
"Issues", the in-game questions in `docs/QUESTIONS.md`.

The phase's manual checks:

TC-SAV-10 manual: checked 2026-10-15, against a stripped copy of the maintainer's save
and a sandbox PES folder (`.tmp/5_5/studio_sl/`): with PES 21 running, sideload compile
exit 0, the patch's `cpks = []`, `savefile_skipped_pes_running`, the copy byte-identical;
with only PES 17 running, `savefile_written` and the copy's `.bak` kept

SE-CLI-02 manual: checked 2026-10-15, with PES 21 running: `import-toml`, `apply-patch`,
`transplant`, `strip-and-seed` and `check-stripped --strip-again` each refused with exit 2
naming PES 2021, no `--out` written; with only PES 17 running each wrote its `--out`
(`.tmp/5_5/cli02/`)

5.4c2 (`strip_base_copied_face`):
manual: checked 2026-10-15, PES 21 Edit mode (`.tmp/ingame/face_check/`): a stripped
base-copied player shows Placeholder; the copied face shows with the face flag clear too

5.5 (the in-game check; runs, shots and saves in `.tmp/5_5/`, by a305229's release exe; the
maintainer's save swapped with their approval and restored, SHA-256 checked):
manual: checked 2026-10-15, PES 21 Edit mode, the maintainer's cup save strip-and-seeded
(`4cc_08_bins.cpk` seeded into the test slot; /d/ 0 of 23 players stripped before, 23 after):
/d/ rows 1-3 and VGL 22 (team 792) rows 5-6 look the same as on the unstripped save
(`pair_d_0N.png`, `pair_vgl_0N.png`), a sample, not every player
manual: checked 2026-10-15, PES 21 Edit mode, stripped save, the /egg/ tracer export compiled
into the test slot: 792's slot 05 shows the compiled full-body model with its boots
(`s3_05_body.png`); the CPK holds `4cc_studio/shared_ids/792.toml` and loads as any other
manual: checked 2026-10-15, same session: slot 06, an `ingame_face` folder whose
`settings.toml` sets only a motion, shows the default face (short dark-brown hair, Face and
Hairstyle editable; `pair4_vgl_06.png`). Without the marker the folder gets the blank face
and no head, as designed (DECISIONS 2026-10-04; `s3_06_body.png`)

5.y-gloves (every `.mtl` of a pre-Fox shared gloves output packed at its root):
manual: checked 2026-10-15, PES 17 Edit mode, /out/ slots 01-03 on shared gloves 1684-1686
(`.tmp/ingame/gloves_check/`, `r3_slot0N.png`, `r4_slot03.png`): `glove.xml`'s
`./grip/glove_r.model` loads; a `.mtl` texture path `./x.dds` and `./sub/x.dds` loads,
`../x.dds` and the game-rooted `model/character/glove/g1684/x.dds` do not (white glove);
a `grip/` model whose `.mtl` sits at the root naming `./x.dds` loads (`r5_slot01.png`)
manual: checked 2026-10-15, PES 17 Edit mode, end to end: run 2's export compiled by
this step's exe (`.tmp/ingame/gloves_check/studio6/`), unpatched: slots 01 and 02 draw their
right gloves magenta (`r6_slot01.png`, `r6_slot02.png`)

### Phase 6 — Export upgrader

Done 2026-10-20. Spec now describes what exists: `docs/spec/export_upgrader.md` (steps 1 to 12,
model-folder mode, the CLI, the message catalog, the Phase 6 acceptance),
`docs/spec/core/development_plan.md` "Phase 6", `team_compiler/testing.md` (the tracers' upgraded cases and the
library parity), `aesthetics_export/object_model.md` (Portraits named `NN`); the plan keeps
step 13 (Phase 7) and the view (Phase 8). Step detail (6.0a-c the open items, the acceptance
section and the Red inventory on `main`; 6.1-6.8 the build, step by step of the plan; 6.9-6.11
the tracers' goldens and parity cases, Red's references recompiled with released Red; 6.12 the
library parity, its census, Red runs and attribution, kept as `scripts/library_parity/`;
6.13-6.16 the defects and the maintainer's rulings it raised: an upgrade writes a clean export
or nothing, leftovers dropped with a Warning, one-wearer item folders' exclusive IDs, Midcup
kits keeping what they do not resend, a pre-Fox Common segment of any name; the converge: the
lead's audit, the design-health passes, Astra and SWE-2 rounds, the citation audits) in git
history up to commit `cf78a8e1`. The close (2026-10-20): the census of every old export on the
maintainer's machine (639 inputs, 279 clean, every unclean run writing nothing, none lost
against 6.16's run), the library parity over the 279 against released Red (every line
attributed but two `red_parity` pairing ties, `.tmp/6_close2/`), the design sweep over the code
since the first close run (`upgrade.rs` past a thousand lines and the staging claim written
twice: `staging.rs`), the citation audit (one flag, EU-REF-12), and the closing whole-crate
mutation runs (extra kinds included, split with bonfire, `.tmp/mutants_6_close2_<crate>/`):
`export_upgrader` 981 caught, 6 missed (the two placeholder views; three equivalents whose
`exclude_re` pins had drifted, re-pinned; a game path's letter case, a test added in
`players.rs`), `team_compiler` 2532 caught, 3 missed (the placeholder settings view;
`match_stem`'s empty-inner guard, a test added; the shared folders' packing sort, an equivalent
pinned in `.cargo/extra_mutants.toml`, since the CPK sorts its entries and the tables their
rows). Carried forward: the placeholder views (8.3), the face transformation for a face made for
the other engine (issue "a face diff is engine-specific", after the DBC editors), the gloves
half of the in-game checks (issue "FoxDen's PES 2021 gloves trampoline"), the `save_editor`
tests a running PES game fails (issue), the in-game questions in `docs/QUESTIONS.md`.

The phase's manual checks:

6.16c (a one-sided hand split, HDG VGL26 slot 22 `EAT`):
    manual: checked 2026-10-19, PES 21 Edit mode (`.tmp/6_16/ingame/`, the export upgraded
    `--loose` and compiled by dc30c45; /hdg/ is the maintainer's save's VGL 33, team 803; save
    backed up and restored): the compile's `GloveList.bin` rows never reach an outfield player
    on the cup's exe (issue "FoxDen's PES 2021 gloves trampoline"; a stripped save, then the
    stock exe, showed stock hands, slot 06's two-sided crossbow gloves included). With the ids
    in the save records (80306 gloves 4186, 80322 gloves 4202, the route the cup uses today):
    slot 06 wears its crossbow gloves (`w01.png`); EAT shows no stock hands, and the split's
    patch floats as a speck beside the model (`w02.png`, `w02_speck.png`)

### Phase 8 — GUI

The phase's steps are itemized when it opens. Before that, its plans get the fixes a survey
found (2026-10-15; the prototype rule and the launcher were fixed that day, DECISIONS):

- [x] 8.0 plan fixes before itemizing (2026-10-18, side-lead C, branch `side/8.0`): the
  "Shell layout" and "Event wiring" sketches now match the spec (tools draw their own panels
  inside the central panel and drain their own sinks in `tick`); the draft-first rule is in
  Phase 8's bullets and each tool plan's build order (`shell.html` first, one page per tool);
  the watcher and the check cache have one home, `libs/pipeline` (DECISIONS); "run strip" is
  the one name (glossary row); `color_tools`' widgets are a Phase 8 bullet, the picker's first
  host the Save editor's color fields (DECISIONS); the Export upgrader plan has a "GUI"
  section; acceptance sections written: `ST` (`plans/core/gui.md`, a new code, CONTRIBUTING),
  `TC-GUI-04` to `-21`, `SE-GUI-01` to `-28`, `EU-GUI-01` to `-06`; TC-GUI-01 narrowed to the
  window and TC-GUI-02/03 automated, cited by the existing headless tests; `eframe` and `egui`
  pinned `=0.36.2` (lockfile unchanged, `cargo tree --locked`); the spec caught up
  (`PipelineEvent::Output` and its paragraph, `pes_process.rs` and the probe paragraph, the
  three registered tools; the plan's registry gained `team_creator`, `ruleset_editor`,
  `db_generator`). The Save editor's spec tree already listed `events.rs`, `diff.rs`,
  `teams_list.rs` (no edit). The upgrader's handoff to the compiler is a question
  (`QUESTIONS.md`). Reviews (checkpoint a; rulings `.tmp/8_0/rulings.md`): SWE-2 V and S1-S6 on
  the branch (S6 4), then on main Astra 8.0.A1 8 accepted, S7 6, S8 6, S9 4, Astra 8.0.A2 3,
  S10 0, every loop ended (2026-10-19, top lead): the colors drawn by storage kind, the team
  colors' 6-bit conversion; a run's events and verdict wake the GUI through the `ToolContext`;
  every close asks every tool (unsaved work, a run in flight), resumed through a `ShellRequest`;
  the GUI's Texport export on PES 18-21; new scenarios SE-GUI-29, ST-RUN-05/06, TC-GUI-22/23,
  EU-GUI-07 (DECISIONS 2026-10-19). Files: `Cargo.toml`,
  `docs/CONTRIBUTING.md`, `docs/GLOSSARY.md`, `docs/plans/core/{gui,architecture,development_plan}.md`,
  `docs/spec/core/{gui,architecture,development_plan}.md`, `docs/spec/libs/pipeline.md`,
  `docs/plans/team_compiler/{README,gui}.md`, `docs/spec/team_compiler/README.md`,
  `docs/plans/{save_editor,export_upgrader}.md`, `docs/plans/libs/color_tools.md`,
  `crates/studio_core/src/shell/mod.rs`, `crates/studio_core/src/pes_process.rs`,
  `crates/tools/team_compiler/src/{view/mod,gui_run}.rs` (citation and pointer comments only)
- [ ] 8.1 egui's bundled fonts, once the theme settles its icon font (`egui-phosphor`): the
  four default fonts are 1.37 MiB of the release exe's `.rdata` (2026-10-16,
  `.tmp/16w/rdata_parts.py`), the two emoji fonts (`NotoEmoji-Regular` 409 KiB,
  `emoji-icon-font` 317 KiB) about 0.7 MiB of it. If the GUI shows no emoji, egui's
  `default_fonts` feature goes off and `Ubuntu-Light` and `Hack` are embedded through
  `FontDefinitions` → verify: every view drawn with the emoji fonts removed shows no missing
  glyph (egui's own widgets draw a few symbols from them), and the exe shrinks by about 0.7 MiB
  (`spec/core/distribution.md` "Size budget")
- [ ] 8.3 carried into the itemizing (2026-10-19): the placeholder `view` and `settings_view`
  of `export_upgrader` and `save_editor` (`lib.rs`, one label each) are the mutants their
  whole-crate runs left missed, deferred because no headless test exists for a label; each
  tool's view brings its own tests (EU-GUI, SE-GUI) and the placeholders go, so the next
  whole-crate run of each crate measures them
- [x] 8.4 the usability pass before the tools' drafts (2026-10-19, the maintainer's request;
  `plans/core/development_plan.md` Phase 8's bullets): after Phase 6 closes and before
  `team_compiler.html`, the lead reads what members already touch (the export format's folder
  conventions, the compiler's and upgrader's message texts, defaults, the upgrade → compile →
  install handoffs) for usability ideas, each a `PROPOSALS.md` entry → verify: the entries are
  written and the maintainer has had them before the first tool draft starts. Done
  2026-10-20: `PROPOSALS.md` P1-P22, curated from 25 candidates a reading subagent gathered
  (`.tmp/8_4/candidates.md`; left out: the face engine marker, which waits for the face
  transformation; a narrower `portrait_conflict`; model-folder mode placing into an export;
  a `settings.toml` template, which the aesthetics editor covers), named in the turn report.
  Files: `docs/PROPOSALS.md`
- [x] 8.5 the Save editor's draft, `docs/prototypes/save_editor.html` (`plans/save_editor.md`
  "Build order and verification"; first of the tool views, for DBC), drawn whole in five
  slices, one per build-order slice, each reviewed before the next is briefed. Drawn before
  the maintainer has ruled 8.4's proposals: the two that touch it (P15, compile findings
  opening the Save editor; P22's `check-aatf` names) add to it and go in once ruled, as a
  slice that changes the draft does → verify: every SE-GUI scenario but the manual ones walked
  in a headless-Edge harness against the page, real sample data (the maintainer's PES 2017
  and PES 2021 saves through `export-toml`), only representable elements
  - [x] 8.5a the page, open and save, the team selector and player list, the info strip's
    quick actions (official, VTL11, DBC7), Abilities & Skills, Appearance (SE-GUI-01 to 09,
    20, 21, 26, 28). Done 2026-10-20 (SWE-2, one rework: two NUL bytes, the sample data one
    player per line, the PES 17 save's placeholder team 701 dropped, the gated controls
    dimmed on screen and the check reading the element): samples from the PES 2017 save
    (702 `/a/`, 718 `/f/`, whose names carry colour codes) and the PES 2021 save (701, 718,
    722) through `export-toml`; harness `.tmp/8_5/run_harness.py`, 15 checks. Drawn for the
    maintainer's eye, each marked in the page: the quick actions follow SE-GUI-09 (one per
    tier, Goalkeeper included) where the sketch shows four; the list's number is the shirt
    number; the Open menu lists discovered saves, then recent files, then the picker; the
    palette swatches' colours are invented (no palette table exists in plan or code), and
    nationality is a number. Files: `docs/prototypes/save_editor.html`
  - [x] 8.5b the Team tab, the team and player operations, FPC (SE-GUI-10, 11, 22 to 24, 27).
    Done 2026-10-20 (SWE-2, one rework): Player, Team and Savefile menus; the boots and gloves
    dialog in 4ccEditor's shape (`bogloDlgProc`); the FPC toggle reading `ops/fpc.rs`'s
    classifier; Output rosters to TSV and Fix database as 4ccEditor's `roster_data_output` and
    `fix_database()` (DECISIONS 2026-10-20, SE-GUI-27 reworded); slice 1's captain checkbox
    off by one fixed. PES 17's kit-ID repair touches container bytes the draft does not model:
    named in the confirmation. Harness 21 checks
  - [x] 8.5c the Tactics tab (SE-GUI-12, 25). Done 2026-10-20 (SWE-2, a fresh session after
    the resumed one stalled at ~20 minutes per turn): the pitch one Painter canvas, markers
    dragged on the stored grid (x 0 to 104, y 0 to 48, 4ccEditor's range on 16 to 21; the
    samples' 42570 slots sit inside it), the slot popover, XI and bench drag lists whose swap
    repoints takers, join-attack and instruction targets, the role palette and join-attack
    assignment modes, the copy/swap menus; slice 1's card badges (per tier, from each ruleset's
    `cards`, `a_positions`, `free` and free-card specials; a badge reads low, never high) and
    slice 2's bound-id width. Drawn for the maintainer's eye: the formation selector stays
    enabled with fluid off (its tooltip gives the game's rule), auto substitution is the
    model's raw setting byte, man marking is carried but not drawn (as in 4ccEditor). Harness
    24 checks
  - [x] 8.5d the comparator, transplant, AATF and patch panels (SE-GUI-13 to 15, 18). Done
    2026-10-20 (SWE-2): the panels as floating windows from a Panels menu, the AATF one also a
    button beside the quick actions; real samples made with the HEAD build
    (`.tmp/8_5/samples/`: `diff` of two PES 17 saves, a transplant and an `apply-patch` written
    to scratch copies and diffed, DBC7's `check-aatf`, two broken rules files); comparator rows
    in the tabs' terms with each field's offset measured on the samples (1-based motions +1,
    physique -7, no row disagreeing); one sentence per violation kind, marked sample wording
    (the CLI prints codes). DBC7 defines no `suggestions`, so its toggle shows none (shown
    under the official ruleset). Harness 28 checks. Three follow-ups ride on 8.5e's brief:
    player names on the panels' rows, the all-teams AATF sample and the comparator's stored
    report lines trimmed (the page grew to 873 KB)
  - [x] 8.5e the interchange dialogs and settings.toml generation (SE-GUI-16, 17, 19). Done
    2026-10-20 (SWE-2, one rework: the harness tail's output line lost in an edit, and every
    team's export text embedded where SE-GUI-16 checks one, the page at 2.5 MB): one import
    dialog for Team TOML, `.4ccs`, `.4cct` and Texport, its sections split as 4ccEditor's squad
    load (DECISIONS 2026-10-20), PES 21 files onto the PES 17 save with the CLI's 189 warnings
    and its two refusals byte for byte, the real `2HU.4ccs` and `IZA.4ccs` decoded with their
    record-to-player pairing (2HU refused on 17: a 20-byte shirt name), Texport export disabled
    on 15 to 17; `settings.toml` generation a player action, coded names written explicitly.
    Slice 4's follow-ups: player names on the panels' rows, the AATF and comparator samples
    trimmed. The page is 1.2 MB, real data: the samples with face and hair values (552 KB) and
    team 718's real export text (265 KB). Harness 31 checks
- Phase 8 opens (itemized 2026-10-20). The shell is Phase 3's slice (a label list and the
  active view, `studio_core/src/shell/mod.rs`); no `ST` or `SE-GUI` scenario is proven, and
  neither `libs/team_widgets` nor `color_tools`' widgets exist. The steps run in DBC's order
  (`plans/core/development_plan.md` "Divegrass Before Christmas"): what the 0.0.1 preview needs
  first, the Team compiler's and the Export upgrader's views after it. Each step builds against
  its draft (`shell.html`, `save_editor.html`) and its plan, and cites its scenarios in tests
  through the shell's headless harness (`shell/mod.rs`'s tests). The drafts wait for the
  maintainer's remarks, so the steps that draw least of what a remark would change come first
  - [ ] 8.6 the shell (`plans/core/gui.md`, `spec/core/gui.md`; draft `shell.html`), sliced
    - [x] 8.6a the sidebar: the planned-tool registry and its greyed entries, which 0.0.1
      also uses for the Team compiler and the Export upgrader (DECISIONS 2026-10-20), the
      logo's About block, the PES selector with its exe check and blocker, the icon rail
      (`egui-phosphor` registered in `theme.rs`), `last_tool` (ST-SIDE-01 to 04, 07,
      ST-RUN-02); the `?` and gear buttons are wired with their windows (8.6d, 8.6c). Done
      2026-10-20 (SWE-2, one rework): `shell/sidebar.rs`, `shell/about.rs`, `theme.rs`, a
      `build.rs` for the target triple, `PlannedTool` and `StudioTool::icon()` in the three
      tools, `egui-phosphor =0.14.0` (egui 0.36's release); the selector's colour one function,
      only files counted as the exe (as the compiler's preflight), the tests' PES folder their
      own. `mutants-diff a720c13`: 41 caught, the tools' `icon()` pinned (`studio`'s registry
      test catches them). Review S1 (SWE-2): 3 accepted of 12, rulings `.tmp/8_6/duck_rulings.md`;
      Astra ran over 8.6a and 8.6b together (A1, in 8.6b)
    - [x] 8.6b the status bar, the window title and the tools' wake callback (ST-BAR-01 to 03,
      ST-TTL-01 to 03), the `data_dir_read_only` condition a failed save raises (ST-SET-03;
      ST-CLI-01), the running-PES auto-switch and its "Fix PES path" notice (ST-SIDE-05,
      manual). Done 2026-10-20 (SWE-2, four reworks): the harness out of `shell/mod.rs`
      (`shell/test_harness.rs`, the sidebar a folder with its `tests.rs`); `shell/status_bar.rs`,
      `shell/title.rs`, the wake slot in `ToolContext` (`emit` and every shell request wake the
      GUI; `request_repaint_after` for a tool's deadlines), the poll in the sidebar (every 5 s
      while unspent, the probe's first game, a blocker deferring it, the action only for a
      folder on this system, `FixPesPath` carrying its version), the CLI's probe and notice
      printer; `Settings::save` edits the file in place (`toml_edit`), and
      `pes_install_missing` is gone (DECISIONS 2026-10-20); a save whose edit would not read
      back as the settings writes the file fresh. The review found a hang: the notice
      printer held stderr's lock, so any warning blocked a CLI command (reproduced, fixed, a
      deadline test). Review: S1-S4 (SWE-2) and A1 (Astra over 8.6a and 8.6b), rulings
      `.tmp/8_6/duck_rulings.md`; `mutants-diff 59216630`: every survivor tested or pinned
      (three pins). Open: ST-SIDE-05's manual check (a PES running, F: connected)
    - [x] 8.6c the settings menu, the first-run dialog and the close hook, sliced
      - [x] 8.6c1 the settings menu: the sidebar's gear, the common rows (the theme applied
        live among them) and one section per tool with settings (ST-SET-01, 02); a
        notice's `OpenSettings` action opens it; its `pes_version` row spends the running-PES
        auto-switch as the selector's pick does; `settings_view` takes the context
        (DECISIONS 2026-10-20). The rows' Browse buttons wait for `rfd`, which the Save
        editor's file open brings (8.9); the tools' own sections are their views' steps
        (the Save editor's `aatf_rules` row with 8.9, the Team compiler's with 8.10). Done
        2026-10-20 (SWE-2, first time): `shell/settings_menu.rs` (a `Modal`, the common grid,
        the sections reset at each opening through `CollapsingHeader::open`), the gear and
        `SidebarClick`, `Sidebar::pick_version` shared by both rows, `theme::apply`
        (`set_theme`, applied in `logic` on change); the lead's fix: the number rows clamp
        only entered values (egui's default clamp rewrote a hand-set `thread_count` on
        sight). `mutants-diff 42f7c7e6`: 2 survivors (`theme_label`), both caught by an added
        assertion. Review S5 (SWE-2) runs over the commit
      - [x] 8.6c2 the first-run data-location dialog (ST-RUN-01) and the close hook for
        unsaved work and runs in flight (ST-RUN-05, 06), which the Save editor's unsaved
        prompt (8.9) and the Team compiler's run (8.10) use. Done 2026-10-20 (SWE-2, one
        rework): `shell/first_run.rs` (a `Modal` with no Cancel while `data_dir` is `None`,
        the choice saved through `Settings::save`, a failed write keeping it open with the
        error; `StudioApp::new` takes the config folder to offer), `StudioTool::hold_close`,
        `ShellRequest::Close` and `ToolContext::request_close` (the close asked in sidebar
        order, the answer back through the window's own close); the shell tests' folders a
        self-removing `TestDir` (they leaked, and three tests shared one); S5's menu fixes.
        `mutants-diff fe5d4af2`: `hold_close`'s default caught by an added assertion, the
        browser stand-in's `up` pinned. Reviews: S6 (SWE-2, 3 of 9 accepted), A2 (Astra over 8.6c1
        and 8.6c2, 4 of 6: a held close closes the settings menu over the prompt, three tests
        made machine-independent), S7 (SWE-2 over A2's fixes, 1 of 5: the About popup closes too), rulings
        `.tmp/8_6/duck_rulings.md`
    - [ ] 8.6d the help window (and the About popup's help link, completing ST-SIDE-06), its
      generated Messages topics and search, `4cc-studio help`
      (ST-HELP-01 to 04), and the log panel widget (ST-LOG-01 to 04); a notice's `OpenHelp`
      action opens the window at its target (8.6b leaves it a dismiss)
      - [x] 8.6d1 the catalogs' member texts (`When`, `Effect`) in each tool's `messages.rs`,
        copied from the spec's tables; the core chapters `setting-up` and `first-run`;
        `4cc-studio help` and `--export` (ST-HELP-04). "Updates" and "What's new" wait for
        the update check and `CHANGELOG.md` (Release 0.1.0). Done 2026-10-20 (SWE-2):
        `CatalogRow` and `HelpSection::messages`, `help::chapters`, `help/export.rs`
        (`chapter_markdown`, `topic_markdown`, `run_help`), `LaunchMode::Help` (clap's own
        `help` subcommand disabled, `help` no tool's id), `Severity::name`; the rows filled by
        `.tmp/8_6/copy_catalog_texts.py` (234 compiler rows, 25 editor, 72 upgrader of which
        47 take the compiler's cells and 18 their code's doc comment), the spec's em dashes
        rewritten. Lead fixes: three upgrader rows whose doc comment split the wrong way, the
        compiler's table moved to `messages/catalog.rs` (data, 1415 lines), the tools' topic
        files' own `#` titles dropped. Files: `studio_core/src/help/`, `shell/launch.rs`,
        `events.rs`, `studio/src/main.rs`, `studio/tests/help_cli.rs`, each tool's
        `messages.rs` and `help/01_command_line.md`. `mutants-diff e45c9672`: 43 mutants,
        `CatalogSeverity::text`'s two missed, caught by an added test of each severity kind's
        spelling. Reviews: S8 (SWE-2, 5 of 10: a spec row wrapped over three lines and its
        copy two spaces short, every compiler row then compared with the spec; a Red state
        file; the upgrader's two shared codes given its own cells), S9 (1 of 4: one of them
        restated), rulings `.tmp/8_6/duck_rulings.md`; Astra waits to batch with 8.6d2
      - [ ] 8.6d2 the help window: `egui_commonmark` `=0.25.0` (the last release on egui
        0.36), the chapter tree, the Messages topic, search, in-doc links, F1 and `?`, the
        About popup's help link, a notice's `OpenHelp` (ST-HELP-01, 02, ST-SIDE-06)
      - [ ] 8.6d3 the log panel widget (ST-LOG-01 to 04) and its code links (ST-HELP-03)
      - [ ] 8.6d4 a member wording pass over the three catalogs' texts, with the maintainer's
        remarks on the draft, to `CONTRIBUTING.md`'s "written for a layman" rule (as easy to
        understand as possible for a member who knows nothing of the code): the copied cells
        carry dispositions (`DropExport`) and pointers
        into `docs/` a member cannot follow, and name Red (the compiler's
        `fmdl_fcl_hair_fallback`, `xml_face_neck_added`, `xml_uniform_pes15`, and
        `player_layout_proto` in both the compiler's and the upgrader's rows)
  - [ ] 8.7 `color_tools`' colour-picker popup (`plans/libs/color_tools.md` Phase 8), the
    Save editor's team colours; the kit menu icons wait for the Team compiler's grid
  - [ ] 8.8 `libs/team_widgets` (`plans/save_editor.md` "`libs/team_widgets`"): the card
    pickers and the violations list first, the tactics editor and its pitch with the view's
    slice 3
  - [ ] 8.9 the Save editor's view in the five slices of `plans/save_editor.md` "Build order
    and verification", with the `session/` wiring each needs (undo, selection, recent files,
    the FPC toggle, AATF quick actions) and the placeholders of 8.3 gone; each slice's SE-GUI
    scenarios cited
  - [ ] 8.10 after 0.0.1: the progress grid, the run strip and the console printer's progress
    line, the exports folder watcher and check cache, cancellation, the launcher, the Team
    compiler's and the Export upgrader's views (their drafts first); the notice action
    `OpenFolder` opens the system file browser for the compile's "Open output" (8.6b leaves it
    a dismiss; eframe's `open_url` opens a browser even for a local path, so not through it),
    and the Team compiler's view stops requesting repaints itself (`ToolContext::emit` wakes
    the GUI since 8.6b); the first tool that publishes a notice during a CLI command gets a
    binary test of ST-CLI-01's notice half (8.6b proves it through the printer's loop only)
- [x] 8.2 the shell's draft, `docs/prototypes/shell.html` (`core/gui.md` "Prototypes before
  implementation"; 2026-10-19, side-lead E, branch `side/8-shell-draft`, SWE-2 in seven
  slices): one self-contained page over the Ruleset editor draft's stylesheet and helpers,
  drawing every screen and state of `core/gui.md` "Sidebar" (the logo's About block with Copy, the
  PES selector's exe-check colours, blocker, once-per-life auto-switch and "Fix PES path", the
  registered tools then the planned ones grayed, the icon-rail collapse), "Status bar" (three
  slots, each item kind's lifetime), "Window title" (activities first, the held notice while
  unfocused, set only on change), "Settings menu" (common rows plus one collapsing section per
  tool, the active one open, best-effort save raising `data_dir_read_only`), "Help window"
  (core chapters then tool chapters, a Messages topic generated from the real catalogs,
  substring search, in-doc links, F1), "Log panel" (cap, paused auto-scroll with "N new",
  filters, chevron, divider, code links to the help, line links to the grid row), the
  first-run data-location dialog and the post-update notice, with a stub central panel per
  registered tool (the Team compiler's holding a simulated compile that feeds the activity,
  the notice, the title and the log). Sample data is real (version 0.1.0, teams-list rows,
  catalog codes and severities, settings defaults); every string left for the maintainer to
  refine is marked `sample wording`. A "Draft controls" strip, visibly outside Studio, flips
  each scenario's GIVEN and ends in a walkthrough naming every `ST` id (drawn, or why not:
  ST-TTL-03, ST-HELP-04, ST-RUN-02..04, ST-CLI-01..02, and ST-SIDE-05's real poll). A
  headless-Edge harness (`.tmp/8_2/run_harness.py`, 29 checks) walks ST-SIDE-01..07,
  ST-SET-01..03, ST-BAR-01..03, ST-TTL-01..02, ST-HELP-01..03, ST-LOG-01..04 and ST-RUN-01
  against the page → verify: each of those scenarios can be walked in a browser with the
  strip's controls, and the page uses only the representable elements (the lead's read of
  every slice's diff against the list). Plan gaps found: `core/gui.md` "Settings menu" lists six
  common settings while `CommonSettings` also carries `check_for_updates` and
  `check_interval_hours` (the draft shows them); the brief's "Fix PES path" example put back
  `**` where `common.rs` means two digits (`20**`), fixed in the draft. SWE-2 review loop
  (rulings `.tmp/8_2/rulings.md`): S1 12/14, S2 7/7, S3 8/9, S4 6/6, S5 5/5, S6 1/2
  accepted, ended at S6; the rounds made the Messages topics the tools' whole catalogs
  (233, 25 and 68 rows, the upgrader's from its `messages.rs` since the plan's table is
  partial by design), one row per code, the source table's own columns; `savefile_path`
  spelled `auto`; no settings section for a tool without settings; the rest wordings and
  draft-control states. For the top lead's plan loop: `update_available`'s Install action
  against 0.1.0's "step 2 as a notice, no Update now"; `ActionEffect` has no variant for
  "Fix PES path" or "Install"; `pes_install_missing` and the selector's yellow report one
  fact twice; the help window's geometry "remembered in common settings" has no
  `CommonSettings` field; `distribution.md`'s changelog heading example carries an em dash;
  two catalog texts a member will read once the Messages topic renders them, the em dash in
  `export_upgrader/src/messages.rs` `common_reference_kept`'s doc and "Red" in
  `spec/team_compiler/messages.md` `player_layout_proto`.
  No Astra round here (checkpoint b on the draft is the top lead's call). To show the
  maintainer; their remarks go into the plan before the shell slices are briefed. Files:
  `docs/prototypes/shell.html`

### Phase 10 — Music tools

The phase's steps are itemized when Phase 9 closes; its format crate was built ahead of it:

- [x] 10.0 `libs/music_export`: the `.4ccm` format crate (`music_player.md` "The `.4ccm` format")
  - [x] 10.0a model, tokenizer, parser, serializer, 7 fixtures, census over the machine's 61
    `.4ccm` (56 parse and read back equal, 5 editor history copies refused for unknown keywords,
    as Rigdio refuses them); `indexmap` enters the workspace; `doc_pointers.py` lists the music
    player plan as open. Files: `crates/libs/music_export/{Cargo.toml,src/{lib,model,tokens,parse,write}.rs,tests/{fixtures.rs,census.rs,fixtures/}}`,
    `scripts/provenance/music_export/list_4ccm.py`, `scripts/doc_pointers.py`, `Cargo.toml`,
    `docs/spec/core/README.md`
  - [x] 10.0b `MatchState`/`Side`/`TeamState`, `Verdict`, `Condition::eval` and `SongEntry::eval`,
    `needs_goal_minute`. Files: `src/{state,eval}.rs`
  - [x] 10.0c `PriorityList::for_slot`/`select`, `Press`, `Loaded`; 9 selection scenario tests.
    Files: `src/select.rs`
  - [x] 10.0d `resolve_song`/`resolve_export`/`Resolution` (native only): the plan's resolution
    order, names matched case-insensitively in the file's own folder and returned as on disk
    (a lead fix: the brief's exact-path shortcut returned the export's spelling on Windows);
    `select`'s skip follows its entry past an unload. SWE-2, first attempt, red runs pasted.
    Files: `src/{resolve,select,lib}.rs`
  - [x] 10.0e the plan's "Model" section carries every type and signature as built (the
    lookup's shapes under "File resolution"), the `pub` audit (`.tmp/10_0e/pub_audit.md`: six
    items with no planned consumer made `pub(crate)`), the design-tell sweep (one triple scorer
    lookup folded, one discarded `Result` made a guard); `just mutants music_export`: 310
    mutants, 282 caught, 26 unviable, 2 timeouts (real infinite loops in `select`'s counter),
    0 missed; the extra kinds' 6 survivors became 4 tests and one simplification
    (`resolve_song` joins the subfolder once), the rerun catches all 26; SWE-2 round S1 4
    accepted (an anchored path's rule named and pinned, the lookup's rules and order stated),
    loop ended; Astra round A1 (top lead, on main) 4 accepted: the `_normalized` companion by
    stem of any extension (IZAKAYA's `.mp3`/`.opus` pairs), the root-level anchor, the
    `advance` skip left out of the `randomise` draw, the `keyword()` sentence narrowed, each
    with a red-run test (DECISIONS 2026-10-18); SWE-2 round S2 on the fix 6 accepted (Unicode
    folding of the companion, `sub/` resolving nowhere, every candidate ranked, a subfolder
    companion test), S3 2 accepted (loop ended); rulings `.tmp/10_0e/rulings.md`. Side-lead B,
    branch `side/10.0e`. Files:
    `docs/plans/music_player.md`, `docs/DECISIONS.md`,
    `crates/libs/music_export/src/{lib,model,eval,state,select,resolve,parse}.rs`

### Phase 16 — Polish and distribution

Steps are itemized when Phase 15 closes; two are fixed already:

- [x] 16.w the release exe's platform floor and size (maintainer, 2026-10-15; taken up right
  after 5.3b3's review, not at Phase 16). A survey of 2026-10-14 (`.tmp/win7/`) found the
  exe cannot load on Windows 7 (Rust std's Win8+ imports and `windows-core`'s `combase.dll`)
  and needs the VC++ redistributable, which no doc names; the maintainer ruled Windows 7 out
  (its last members moved to Linux). Slices:
  - [x] 16.w1 `docs/spec/core/distribution.md` states the floor, Windows 10 x64 (and Linux),
    with the reason (the Rust target and the GUI stack require it) → verify: `just doc-pointers`.
    Done: the "Platform floor" paragraph, DECISIONS 2026-10-15
  - [x] 16.w2 `+crt-static` for the Windows release build (measured +327 KiB, 1%), so the exe
    runs without the redistributable → verify: the release exe's import table has no
    `VCRUNTIME140.dll` and no `api-ms-win-crt-*` (`.tmp/win7/imports.py`); the GUI starts.
    Done: `.cargo/config.toml` (every MSVC build); 7 CRT imports before, none after; +225 KiB;
    the GUI started hidden, alive and responding with its window after 8 s
  - [x] 16.w3 the size growth (17.1 MiB on 2026-10-02 to 30.95 MiB on 2026-10-14): find its
    cause with `cargo bloat` (approved as a developer tool), make the low-hanging reductions
    that also shorten builds, and update the spec's measurement → verify: before and after
    sizes and clean release build times, measured the same way. Done (DECISIONS 2026-10-15):
    the growth is features (`.tmp/16w/`: the 2026-10-02 commit's crate table against today's);
    `rhai` `only_i64` -627 KiB; 32,723,968 bytes and 116 s before, 32,311,808 bytes (static
    CRT included) and 124 s after, one clean build each; the spec's "Size budget" updated

- [ ] 16.x `dds_convert` GPU BC7 (deferred from 2.5b to Phase 4, decision entry 2026-09-21, then
  past Release 0.1.0, decision entry 2026-09-28; spec `libs/README.md` "First-release desktop GPU
  BC7"): `block_compression`'s wgpu backend on a Vulkan/Metal device, CPU fallback with the
  fallback reason reported, cold pipeline creation and upload/readback measured first, then the
  texture step's bounded batches → verify: GPU and CPU outputs decode within the same tolerance
  on the `dds_convert` fixtures; the fallback path exercised by forcing no adapter
  - [x] 16.x a `dds_convert`'s half, built ahead and unwired (2026-10-19, DECISIONS "the GPU
    BC7 encoder is built ahead of 0.1.0"; measurements `.tmp/16x/findings.md`: Vulkan 0.48 s
    per 4096 chain against 16.1 s on 16 CPU threads, same PSNR; DX12 refused): `gpu::GpuEncoder`
    (Vulkan/Metal, 4-Mpixel slabs per submission, error scopes, a device failure sends that mip
    and everything after it to the CPU, a texture past the device's limit goes to the CPU
    whole), `convert_with_gpu`, `pad_to_blocks` shared, a wasm stand-in; `just gpu-tests` runs
    the `#[ignore]`d GPU tests (CONTRIBUTING); mutation run over the diff with them
    included, survivors tested or excluded (`.cargo/mutants.toml`). Files: `Cargo.toml`,
    `crates/libs/dds_convert/{Cargo.toml,src/gpu.rs,src/encode.rs,src/lib.rs}`, `justfile`,
    `.cargo/mutants.toml`, `docs/CONTRIBUTING.md`, `docs/plans/libs/README.md`
  - [ ] 16.x b the Team compiler's texture step on the encoder, after 0.1.0: the CPU starts at
    once and the GPU joins when its device is ready, started when the run has BC7 work
    (`plans/libs/README.md` "First-release desktop GPU BC7"); start it after the window's
    renderer is up, a failed start this run's CPU fallback with its reason in the report
    (`.tmp/16x/findings.md`, the transient adapter failure seen under load)

### Phase 20 — Ruleset editor (post-release)

The phase is itemized when it opens; its GUI is drafted ahead of it, at the maintainer's request
(2026-10-14), to show the members who offered to help shape the ruleset format:

- [x] 20.0 The Ruleset editor's GUI draft, right after 5.2c3-5.2e1 land with their review
  rework, before 5.2e-2's VGL/VTL rulesets (moved up by the maintainer, 2026-10-14): the draft
  needs only the schema, and input that changes the format costs less before more rulesets are
  written against it. Its purpose is to show the members that a Rhai rules file is a sound
  choice: the GUI makes it easier to edit than a simpler hand-written text format would be,
  while the file stays readable. So beside the forms the draft shows the ruleset's data block
  read-only, updated as a field changes.
  - [x] 20.0a coverage of a contributor's proposal (Clouds, written without knowing Studio;
    the maintainer's copy is `.tmp/Clouds_proposal.txt`): a plain-language rule system with
    tier and bracket groups (`medal` for every medal tier), player variables (each stat, card
    and COM style, allowed A positions, height, weight, age, strong foot, weak-foot accuracy
    and usage, injury resistance, form, registered position, playing style), per-preset
    tactical variables (each player's position and x/y per preset and fluid formation),
    `IF`/`ELSE`/`AND`/`OR`/`NOT` conditions with comparisons on any variable, additive `_mod`
    modifiers (each stat, all stats, A positions, the height allowance capped below the next
    bracket, weak foot, injury resistance, form, card limit) and `OR` between modifiers tried
    as needed before the card limits. Not adopted as a format; the check is that the
    `aatf_rules.md` schema can express everything it can, field by field, each gap either
    added to the schema (plan edit, decision entry) or answered by a custom check, with the
    reason → verify: a table in `aatf_rules.md` mapping each of the proposal's items to a
    schema field or to a custom check, no row empty. Done 2026-10-14 (lead): `aatf_rules.md`
    "Coverage of a plain-language proposal", 30 rows; the gaps became fields (`ranges`,
    `strong_foot`, `any`, the `form` and `injury_resistance` effects, `team.formation` for
    custom checks), implemented at 5.2d2; `ruleset_editor.md`'s forms carry them
  - [x] 20.0b the interactive draft (`core/gui.md` "Prototypes before implementation"):
    `docs/prototypes/ruleset_editor.html` over the `ruleset_editor.md` forms, the official
    Autumn 26 ruleset as its sample data, every schema field reachable, the data-block pane
    beside the forms → verify: each of `ruleset_editor.md`'s editor sections has its form in
    the draft, each row of 20.0a's table that maps to a schema field has a widget the page
    can show it in, and the page uses only the allowed widgets (the lead's read against that
    list). Done 2026-10-14 (lead): one file, every section of the
    editor (start page, the nine sections, undo and redo, the load rules as live problems
    with the rail's marks, Save refusing while a problem stands); the data block is rebuilt
    on each edit, changed lines highlighted, and for the unedited sample holds the same
    values as `rulesets/autumn26.rhai` (a headless-Edge harness, `.tmp/20_0/run_harness.py`,
    which also renders every section and adds every condition and effect kind). Randomize,
    Open and the other templates are shown disabled. Per-group free cards moved into each
    tier's expander (`ruleset_editor.md` "The editor"). To show the members; their remarks
    go into the plan before Phase 20's itemization
  - [x] 20.0c the draft brought up to date (maintainer's request, 2026-10-15): the five other
    embedded rulesets as working templates (custom checks included), `problems()` covering
    every `LoadRule` (`GroupsOverlap`, `ReservedGroup`, `UnknownTarget`,
    `SystemConditionTooWide`, ids inside effects; a name colour is eight bytes, any bytes), and
    a sixth template, "Clouds' proposal, every item (from VGL26)", using each of 20.0a's 30
    rows (each special's label names its item; the two custom-check rows as a real section
    over `team.fielded_positions` and `team.formation`) → verify: the harness shows every
    template valid with every view rendered, each transcribed block equal to its `.rhai`, a
    red case per new rule; the Clouds block assembled as a rules file loads and runs in
    `check-aatf`. Done 2026-10-15 (SWE-2; the lead cut the Clouds height quotas to giga,
    giant and manlet, which every legal team meets, and moved three stat bonuses off stats a
    tier already holds at 99, whose targets no player can reach). Run over the VGL26 cup save's
    teams 781 to 834, VGL26's own file reports the 2 known violations; the Clouds file places
    every tier and reports only what its own specials ask for (form 8, goalkeeping 85, buffed
    stats 87). Files: `docs/prototypes/ruleset_editor.html`
  - [x] 20.0d coverage of the DBC ruleset (maintainer, 2026-10-19), right after Phase 6
    closes (`plans/core/development_plan.md` "Divegrass Before Christmas"): Divegrass Before
    Christmas (DBC), an upcoming invitational, is to be the first test ground for 4cc Studio's
    Ruleset editor and Save editor. Its rules' source of truth is the wiki page
    `https://implyingrigged.info/wiki/Divegrass_Before_Christmas_7:_Melty_Under_The_Christmas_Tree`
    ("Tactical export Rules": height brackets, medal systems; the game is PES 2017), read
    through the `stealth-browser-mcp` server (a plain fetch gets HTTP 403) and re-read when the
    step starts (snapshot of 2026-10-19: `.tmp/dbc7/wiki_rules_2026-10-19.md`). How they are
    checked is in `C:\Data\4cc\Tools_4cc\DBC7-Editor`, a small fork of the official 4cc save
    editor (C++, `RockAnon/DBC7-Editor`; the rules in `aatf.cpp`, its commits "dbc7 specific
    aatf and buttons" and "multi gk and no gk stuff" over the initial import), a reference
    where it and the page differ. Every DBC rule gets UI-class support in the Ruleset editor (a
    schema field with its form), except one niche and complex enough to be a custom check,
    each such with its reason → verify: a table in `aatf_rules.md`, as 20.0a's, mapping each
    DBC rule (read from the page, with the fork's diff against the official editor for how it
    is checked, not from memory) to a
    schema field or to a custom check, no row empty; the gaps become schema fields (plan edit,
    decision entry) and forms in `ruleset_editor.md` and the draft; a DBC ruleset file in
    `rulesets/` that `check-aatf` runs against a save the fork's check agrees with. Done
    2026-10-20 (SWE-2, lead fixes): the page re-read (unchanged since 2026-10-19); three schema
    fields, none a custom check (`plans/aatf_rules.md` "The ruleset schema", "Coverage of the
    DBC7 rules"; DECISIONS 2026-10-20): `medal_systems` (GGG, GGSS, GSSSS, SSSSSS),
    `HeightSystem::by_quotas` (a system told apart by its counts) and `extra_height` (GigaHeight's
    50 cm pool, at most 210), with the violations `MedalSystem` and `ExtraHeight` and their load
    rules; a tier is recognized before its choices (DECISIONS 2026-10-20), which the parity run
    found (a Manlet silver at 92 read as a regular); `rulesets/dbc7.rhai` and its fixtures; the
    draft's forms, load rules, summary and a DBC7 template. Parity against the fork's
    `aatf_single`, built headless, on the maintainer's two PES 2017 saves (220 teams each):
    every difference a listed class (`plans/aatf_rules.md` "DBC7"); the fork passes 29 and 36
    teams that the block fails on stats inside the fork's warning band alone (QUESTIONS "DBC7:
    the organizer's editor errs on what the page leaves free"). The draft's dumped block runs in
    `check-aatf` with the same output as `dbc7.rhai`. Reviews: SWE-2 S1 (3 accepted), Astra A1
    (2), SWE-2 S2 (1) (`.tmp/20_0d/duck_rulings.md`). Files: `crates/libs/aatf/` (`logic/
    logic.rhai`, `rulesets/dbc7.rhai`, `src/ruleset/{mod,check,errors}.rs`, `src/violation.rs`,
    `src/file.rs`, `src/fixtures/`, `src/ruleset/tests/`), `crates/tools/save_editor/src/
    messages.rs`, `docs/plans/aatf_rules.md`, `docs/plans/ruleset_editor.md`,
    `docs/prototypes/ruleset_editor.html`, `docs/DECISIONS.md`, `docs/QUESTIONS.md`

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history. A question only the maintainer
can answer is not an issue: it goes to `docs/QUESTIONS.md`, and an issue that waits on one
points there.

- open 2026-10-20 — two `save_editor` CLI tests (`strip::tables_the_bins_lacks_are_created_from_the_seed`,
  `strip::the_tables_are_seeded_and_every_appearance_id_is_set_to_minus_one`) fail while
  PES 2019 runs on the machine: the command's running-game refusal (`src/cli.rs`, "is running:
  it writes its own save back") reads the real process list, so a gate run during an in-game
  check fails (6.16d's first run). Likely fix: the tests inject the process probe (or the
  command takes it from its context), so no test depends on what else the machine runs.
  Until then, close the game before `just gates`.
- open 2026-10-19 — the maintainer saw AMD driver timeouts during the BC7 encoding tests
  (`dds_convert`'s `gpu.rs` tests, 16.x a). Logged so far: one Windows GPU reset (System log,
  event 4101 "Display driver amduw23g ... stopped responding and has successfully recovered")
  at 03:16:41, while 16.x's probe (`.tmp/16x/gpu_probe/`, edited 02:50-03:09) dispatched whole
  mips in one submission, before `gpu.rs`'s bounded slabs (`SUBMISSION_PIXELS`, c68ef99 at
  04:36); none logged since. Still to rule out: the tests' own load, each GPU test building its
  own device and `cargo test` running them in parallel, several worktrees' gates at once, a
  `dds_convert` mutation run's parallel jobs. Parked (the maintainer, 2026-10-19): no
  investigation now; during every later BC7 test (16.x b, a `dds_convert` mutation run) check
  the System log for event 4101 (`Get-WinEvent -FilterHashtable @{LogName='System'; Id=4101}`)
  against the run's times, and if one recurs, reproduce (the GPU tests looped, alone and in
  parallel processes) and serialize the tests' device use or lower the budget, whichever the
  measurement points at.

- resolved 2026-10-17 (after Phase 5's close; `scripts/mutants.py` `plan_file_split`) — a split mutation run waits on the
  VPS for crates whose tests dominate: the halves are equal (`--shard 0/2` and `1/2`), but with
  half this PC's threads per test (8 vCPUs against 16) the VPS tests `aatf`, `save_editor` and
  `pes_savefile` at 60-80% of the local rate (5.6 runs: 92 against 157 an hour, 163 against 248,
  84 against 107), while build-bound crates run at par or faster. Give the local side the larger
  share in proportion to the two sides' measured rates per crate (`pes_savefile` about 55/45,
  saving about 1.4 h of its 11.5), only if it stays exact: cargo-mutants takes one shard per
  invocation, so the share needs either several local shards of an n-way split run one after
  another (one extra baseline each, about 75 s for `pes_savefile`) or another way to hand each
  side its list; whichever is used, the merged outcomes must name every listed mutant exactly
  once (checked against `--list` in the summary, a run failing it reported, not summarized).
  `scripts/mutants.py`; RUNBOOK "Mutation runs and the VPS". Resolved: a whole-crate run
  hands each side a complementary set of source files (`examine_globs` here, `exclude_globs`
  there) sized by the two sides' measured mean seconds per mutant, checked exact against
  `--list` before launch and against the merged outcomes after (exit code 5 otherwise);
  round-robin stays for diff runs and unmeasured crates. The remote means were seeded from the
  close runs' archives (`.tmp/5_6/shards/seed_remote_costs.py`); `studio_core`'s first file
  split put 38 of 85 mutants here, the halves ending 146 s and 153 s in.
- open (2026-10-15, 5.6c census) — PES 2014 files read as PES 15 and then fail with a
  misleading error: `SaveContainer::decrypt` labels a PES 2014 `EDIT.bin` or `TEXPORT0N.bin`
  `Pes15`, and the decode fails on a player name (not UTF-8) or a tactics byte. Loud, never a
  wrong write, but it should be refused as PES 2014 (unsupported): measure what separates the
  two versions' containers (PES 2014's EDIT is 4,799,651 bytes, its texport 5,389,806; no real
  PES 15 texport is on the machine) and refuse it by name. `.tmp/5_6/audit_notes.md`.
- open (2026-10-16, 5.6b mutation runs) — three GUI views are placeholder lines no test reads:
  `save_editor`'s `view` and `settings_view` and `team_compiler`'s `settings_view`
  (`lib.rs`), each a survivor when replaced with `()`. Their tests come with the views they
  stand in for (the Save editor's window, the Team compiler's settings editor, Phase 8), with
  `team_compiler`'s headless `view/mod.rs` harness.
- open (2026-10-15, Astra review B.A1 of 5.3b) — nothing refuses an `output_folder_path` that
  resolves to the PES folder's `download/` (or inside the PES install): a degraded or
  `--no-deploy` run then installs into the game past every deployment check, and a savefile
  failure's rollback promotes the new CPK over the old one it just restored. Fix when the
  settings get their validation pass (Phase 8's settings view at the latest): refuse such a
  path on the command line and in the settings view, comparing the canonical paths.
- resolved 2026-10-14 (5.2f-r2) — every team of the VGL26 cup save fails VGL26's "Support settings"
  special on `auto_preset_change` (all three PES 21 cup saves hold it off): the rule follows the
  rules page, which speaks of the deadline export. Resolved the same day (step 5.2f-r2): the
  page's "switch preset tactic" is the tactics menu's option, not the byte, which VGL26 and
  VGL27 now require off.
- open (2026-10-11, Phase 5 Acceptance; deferred until the cup next plays a pre-Fox version) —
  the pre-Fox aesthetics patch (`pes_savefile/operations.md` "Aesthetics patch", the pre-Fox
  half; `spec/team_compiler/pipeline.md` "Post-processing"): every appearance field, the resolved boots and gloves
  IDs through commit-conditioned mutations, the edit flags and the base-copy ID (issue below).
  Until then a compile for PES 15-17 writes no patch and leaves the savefile alone (decision
  2026-10-11). With it, lift the Fox-only gates of `team_compiler` `validation.rs`
  `settings_id_messages` and `settings_name_messages` (their catalog rows say Fox only).
- open (2026-10-11, Phase 5 Acceptance) — a referee export's `settings.toml` reaches no table or
  patch: a referee's look is the template's `RefereeAppearance.bin` (`blue_port.md`), whose rows
  nothing writes, and `blue_port.md` has `fpc_off` apply its settings preset to a referee on
  PES 18-21. Settle with a Fox refkit (the issue on a referee's body under `fpc_off`): which
  table a referee's settings go to, and TC-REF-19's outcome then; with it, lift
  `settings_id_messages`' team-only gate (`validation.rs`).
- open (2026-10-11, 5.0c; deferred until the cup next plays a pre-Fox version) — what Red does
  on PES 15-17 that Studio does not (`.tmp/5_0c/xml.md`, `checks.md`), most noticed first:
  - a member's `face.xml` that does not list `hair_high_win32.model` (1,248 of 1,263 such
    folders on the maintainer's machine): Red packs the file and the game loads it by its fixed
    name (inferred), while Studio reports `xml_model_unlisted` and leaves it out, so the head
    compiles bald
  - a generated `face.xml` lists `hair_high` as a `parts` entry, while Red leaves
    `hair_high_win32.model` out of the xml and packs it under its own name; the in-game effect
    is unknown
  - a `.mtl` Common path that only an installed CPK supplies keeps its `XXX`, while Red's
    `txt_id_change` writes the team ID in every `.mtl` (the answer to the question
    `messages.md` leaves for the pre-Fox parity step)
  - a Common model or `.mtl` that a member's xml names and only an installed CPK holds: Red
    accepts it, Studio reports `xml_model_not_found`
  - a WESYS-wrapped `face.xml` (48 on the machine): Red unwraps it, Studio reports `xml_broken`
  - a `.mtl` Common path whose team folder is not 3 characters: Red reports an Error, Studio
    nothing
  - a `<model>` with no `material`: Red reports an Error, Studio writes it (TC-XML-01)
  - `mtl_state_nonrecommended` misses Red's `alphablend 0` with `zwrite` other than 1 (Info)
  - two models packing to one name (`hat.model` beside `oral_hat_win32.model`)
  - the dummy team-ID respelling of file names (`Common/u0XXXg1.dds` named as
    `.../common/XXX/u0XXXg1.dds`, 9 exports): the file and the path are respelled together or
    not at all
  - DECISIONS 2026-10-09 sends the xml's `<dif>` on PES 15-17 beside a `face_diff.bin`, while Red
    lets the `.bin` replace it; the two differ in 36 of 53 such folders
  - `boots_edit.model` and `glove_edit.model`, the Edit-mode models (2 found), and a member's
    own `glove.xml`
  - a member's `face.xml` path in a game-path form other than `model/character/face/common/…`:
    Red reports an Error for a model or `.mtl` (a Warning for a texture), Studio keeps it with
    `xml_path_unchecked` (W); DECISIONS 2026-09-12's "Red waved unresolvable game paths through
    silently" holds for that one form only (`messages.md`, the `xml_path_unchecked` paragraph)
- resolved 2026-10-14 (maintainer: `Tier::name_color`, written by `apply_tier`; `aatf_rules.md`
  "The ruleset schema", "The host"; implemented at step 5.2d2) — `apply_tier` cannot make a VTL11
  medal legal: VTL11 requires a medal's name to carry a gold or silver colour code (a
  `name_colors` condition), which no tier target expresses, so the quick action writes no name
  and the checker then rejects the player, against `aatf_rules.md`'s "neither tool can produce
  a player the checker then rejects". The VGL fork's editor writes the colour itself (`9951666`,
  and `vgl27-Ruleset-qol`'s `auto_color_names`). Recommended: a tier target for the name colour,
  which `apply_tier` writes (and strips from a tier that forbids one), over a separate action.
- open (2026-10-11, 4.y-mattable row 27; deferred until the cup next plays a pre-Fox version) —
  the cup wiki says a `.mtl` splits a path at spaces, and 212 of 60,027 texture names in real
  FMDLs hold one (`Kit 1_u0XXXp0.dds`), so a Fox model converted for PES 15-17 may write a
  `.mtl` path the game cuts short. Unverified in game.
- open (2026-10-11, 4.y-fix23 (1); deferred until the cup next plays a pre-Fox version) — a
  PES 17 referee whose only model is a `.model.common` link gets no face (`roles.rs`
  `is_package_model` does not count `PreFoxCommonModel`), while the converted Common model is
  still written into the refs CPK with nothing naming it; the link should count as his face
  part, as a team player's does.
- open (2026-10-11, S19.1-B3) — which material files a glTF in a player or shared folder's
  subfolder reads (`model_format.md` "Name matching" is written for one flat folder; the
  `.mtl` search's two folders, the file's then the model folder's, are the likely answer):
  for Phase 7's plan pass, when glTF sources are planned.
- open (2026-10-09) — a Fox referee's body under `fpc_off` waits on a Fox refkit, an FMDL
  body the maintainer will make ("We'll make one later"; `blue_port.md` "The referee
  body"): then the Fox referee template ships it and a step merges it into an `fpc_off`
  referee's face package. Until then `fpc_off` applies its settings preset alone on Fox.
- open (2026-10-09) — per-kit face packages on Fox wait on FoxDen (`model_format.md`
  "Kit-dependent assets", DECISIONS 2026-10-09): each kit number gets a face package of its
  own, merged from that number's variants of any file kind, once FoxDen loads one per kit
  picked; the layout it reads is decided with FoxDen's work, then a step in the phase open
  at the time builds it and retires `kit_variant_model_fox`. FoxDen is a work in progress.
- open (4.y-fix17's review, 2026-10-10) — a PES 17 `.model` collar compiled for PES 15 or 16
  keeps its `uni_collar` material, packed as is when the pre-check does not flag it (PES 16)
  or moved onto the skeleton when it does (PES 15), while those versions' `uniform.mtl`
  defines no `uni_collar` (the FMDL conversion names `uni_shirts` alone for them, DECISIONS
  2026-10-09). What the game draws for a collar material its `uniform.mtl` lacks is unknown:
  an in-game check on PES 16 (the F: install), and if it draws nothing, the same-engine
  collar gets the version's names as the converted one does.
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

- open (found at 4.26c's review) — when an all-or-none install's undo cannot move an old
  CPK back from `.cpk.old` (`deploy.rs` `undo`), the failure is only logged: `download/`
  then lacks a CPK its list names, so the game loads none of the folder, and the member
  sees only the `old_cpk_locked` or `deploy_target_unwritable` that started the undo. The
  move back renames a file the run itself just moved, so nothing real is known to fail it;
  the plan names no finding for it. Revisit if a member meets it.

- open (found at 4.26b's review) — a multi-CPK run promotes its CPKs to `output/` one
  rename at a time, so a rename failing part-way leaves `output/` with some new CPKs and
  some of the previous run's. The plan settles all-or-none for the installation only
  (`spec/team_compiler/pipeline.md` "Deploy CPKs"); the output folder is the maintainers' build folder, and the
  next run rewrites every CPK. Revisit if a maintainer meets it.

- open — the `livecpk/` replacement is not atomic (4.23a): the previous tree is removed before
  the staged one is renamed in, so a copy that fails partway (the output folder on another
  drive) or a removal that fails partway (a file the runtime holds open, unverified for FoxDen)
  leaves `livecpk/` partial or empty, reported as `output_commit_failed`. A swap through a
  sibling folder would narrow it; decide with the first in-game sideload test.
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
- resolved 2026-10-15 (Phase 5 converge: step 5.3a writes the default-ID rows, TC-BIN-25
  and 27; the gloves' look waits on the FoxDen issue below, "FoxDen's PES 2021 gloves
  trampoline") — the boots/gloves rows a `Full` export leaves (`pipeline.md`
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
- answered (2026-10-05, below) — the seed rows and multi-CPK mode: strip-and-seed puts its rows
  into `4cc_08_bins.cpk` (decision 2026-10-05), but in multi-CPK mode that CPK is the
  compiler's own bins output (`settings.md` `bins_cpk_name`), and the bins walk starts below
  the output CPK (`spec/team_compiler/pipeline.md` "Bins accumulation"). So the DLC builder's next multi-CPK
  compile reads the three player tables from below the seed and replaces `4cc_08_bins.cpk`
  without the seed rows of every player no export compiles. Resolved (2026-10-05,
  maintainer): in multi-CPK mode the player tables' walk starts at the bins CPK itself
  (`spec/team_compiler/pipeline.md` "Bins accumulation").
- answered (2026-10-05, below) — motions on Fox: `settings.toml`'s `[appearance.motion]`
  (hunching, arm movement, kick motions, celebrations, dribbling) are player-record fields
  outside the appearance block (PES 20/21 bits 96 to 332), so neither the `PlayerAppearance.bin`
  row (the block's bytes) nor the Fox patch (names only) carries them: on Fox a compiled
  export's motions reach nobody. Resolved (2026-10-05, maintainer): the Fox patch carries
  every compiled player's motions (`pes_savefile/operations.md` "Aesthetics patch"). Untested: that the game
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
  models"; `kit_variant_model_fox` and `spec/model_format.md` "Kit-dependent assets" change with
  the answer.
- resolved 2026-10-12 (DECISIONS "the ingame-face run's undecoded bits travel between
  versions") — the unknown bits across versions: per-version tables by record bit position
  dropped hair and sliders when a PES 16 export compiled for PES 21; the run's bits are now one
  table by position in the run, written on every version (review S5.A1 item 8).
- open (deferred with the pre-Fox patch, decision 2026-10-11) — the base-copy ID on pre-Fox:
  `model.md` makes it compiler-owned ("derived from what
  was written"), but no plan says what the compiler derives, and the aesthetics patch, the
  only savefile write path, has no key for it (Team TOML has `base_copy_id`). Settle with
  the pre-Fox patch: the rule and its key, with the edit flags the patch writes (review S5.A1
  item 10).
- resolved 2026-10-15 (maintainer: a Midcup carries only the changed shared folders; DECISIONS
  "a compiled CPK records its shared IDs, and a Midcup keeps them"; step 5.3a builds it) —
  shared IDs and retained rows: a team's shared boots/gloves IDs are assigned in
  alphabetical folder order from the export being compiled (`player_folders.md` "Assigns IDs
  automatically"), while a player the export does not compile keeps his installed row
  (`spec/team_compiler/pipeline.md` "Bins accumulation"). A `Midcup` export holding shared folder B but not A
  gives B the ID A had in the full CPK, so A's wearers, their rows untouched, now wear B.
  The same holds for pre-Fox save IDs. Settle before 4.22: a midcup keeps the full export's
  assignment (from where?), or must carry every shared folder its team uses (review S5.A2
  item 1).
- resolved 2026-10-17 (DECISIONS "generation, portraits and the CLI settled"; plan step 8:
  the upgrader generates from the record) — generation from a stripped save: `settings_toml.md` has generation skip a player
  at -1, so an old-format export migrated against the cup's stripped Fox save gets no
  `settings.toml` for him and his next compile writes all-default rows over the look the
  tables hold. Reading his installed table rows (the bins walk) instead would keep it.
  Settle with the Export upgrader's generation, Phase 6 (review S5.10 item 3).
- resolved 2026-10-17 (same decision: no `fpc_on` marker is written, raw settings and the
  players' own models reproduce them) — the Export upgrader's `fpc_on` detection (`export_upgrader.md` step 8, "settings
  match the enable preset"): the cup's FPC players mostly ride their bodies in per-player
  boots IDs, so their boots field holds that ID, not 55, and Test 1's save had every /a/
  player FPC while `is_fpc_player` found none (`.tmp/4_0/apptest/results.txt`, run 2). Matching
  the strip fields alone would also mark a dressed player with long sleeves, tucked shirt and
  short socks. Settle the fields compared before Phase 6 (review S5.11 item 2).
- resolved 2026-10-17 for generation (same decision: a version's unmeasured range is its
  field's bits, for generation and the compiler's narrowing, `plans/aesthetics_export/settings_toml.md`;
  built in Phase 6) — real cup players past the editor ranges: `PlayerSettings::from_player` refuses a
  stored value its key's kind cannot represent (`OutOfRange`, documented as "a hostile save's
  out-of-range bits"), but the PES 18 fixture save holds 79 such players and the PES 19 one 26
  (`cheek_type` 4 to 5 where the range ends at 3, `forehead_type` 6 where it ends at 5, inside
  the fields' bits; found at 5.1b). So a `settings.toml` cannot be generated for them, and
  strip-and-seed, whose rows must equal each player's record bytes (SE-STR-01), cannot take
  the `from_player` route: `appearance_row` itself writes such a value as is (the field's bits
  are its bound), but the Save editor has no public way to build those settings. Settle before
  5.4c (a row straight from a `PlayerEntry`, or wider ranges if the game shows those values),
  and before any tool generates `settings.toml` from a save. Strip-and-seed's half settled
  2026-10-14: its rows come from the record (`EditFile::appearance_row`, step 5.4c1); open for
  generation only, settled with the Export upgrader's generation (Phase 6).
- resolved 2026-10-15 (in-game check; DECISIONS "a base-copied face is any base-copy id other
  than 0 and the player's own"; step 5.4c2 builds the warning) — strip-and-seed and a
  base-copied face, measured 2026-10-14 (step 5.4c; census
  `.tmp/5_4/base_copy_census.txt`, plan "Stripped save"): base-copied faces are reported, not
  carried. The issue: the `PlayerAppearance.bin` row carries no
  base-copy id (`spec/pes_savefile/model.md`, "Player settings model (settings.toml)"), which holds a face-transplant
  relationship (`spec/pes_savefile/operations.md` "Save-to-save operations"), so a stripped player whose
  record copied another player's face may change face, against "stripping changes nobody's
  look". Untested (Test 1 did not cover it). Measure before strip-and-seed is built; then
  carry the relationship or report the affected players (review S5.A3 item 3).
- open — an unwritable `livecpk\`: sideload mode writes it "with the same access path as
  deployment to `download/`, elevation included" (`spec/team_compiler/pipeline.md` "Output modes"), but no
  plan says what a sideload run does when it cannot (a tree has no `output/` to degrade
  to). Settle with the sideload step (review S6.1b item 21).
- open — the deep pass reads everything `compile` reads again (4.7): each checked file is
  read once by the pass and once by its task, and a solid `.7z` is decompressed twice (its
  check, then its tasks; three times before 4.7f for an export holding a metadata file).
  For a folder the second read comes from the system's file cache; for a `.7z` it is a whole
  extra decompression per export, about 2 s for a 0.6 to 1 GB export (4.7f's timing): about
  2 s of a 5.2 to 6.8 s compile. The buffer is not kept from the check to the tasks because
  planning needs every export validated first, and holding every `.7z` until then is the
  residency the budget forbids (`spec/libs/pipeline.md` "What a solid `.7z` is charged"). The
  maintainer's answer (2026-10-10): a small archive is kept, step 4.y-7z, done the same
  day: the `.7z` half of this issue is closed; the folder half (the file cache's second
  read) stands.
- open — a face diff is engine-specific (maintainer, 2026-10-03): a diff authored for one
  engine misplaces the face on the other; the compiler passes a diff through for whatever
  target it compiles, so a diff is the author's responsibility, as with Red. Investigated
  offline 2026-10-15 (`.tmp/face_diff/findings.md`): one format on both engines, Konami's
  diffs carried unchanged from PES 17 to 18; the real split is version 4 (32 entries) against
  5 (33), with a mechanical conversion. The maintainer's example (2026-10-17, a member's
  anime face, `.tmp/face_diff/lapwing/findings.md`): the Fox fix is two eye floats (head 0x2C
  and 0x30, 35 to 45), which PES 21's Edit mode does not show even at 0 and 90. Settled
  2026-10-19 by a second example and its author's explanation (`Sopdog`, the engines animate
  eyes differently, so model and diff are made per engine and do not convert). Answered
  2026-10-20 (DECISIONS "a face made for the other engine is transformed"): a `fox`/`pre-fox`
  face marker and the Sopdog pair's transformation applied when the engine changes, attempted
  after the DBC editors (plan `model_conversion/conversion.md` "A face made for the other
  engine"): derive the rule from the pair, check it in game on both engines, then build it.
- open — u16 face indices cap a reassembled split mesh (found at 2.20f review): `fmdl::Mesh`
  and the IR (`spec/model_conversion/ir.md` "IR struct") store faces as `[u16; 3]`, so `fmdl::ops::split::decode`
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
- open — `pes_savefile` the kit-slot rows: the ten-slot kit block is measured on 18 (+0x1C),
  19 (+0x20) and 21 (+0x30), with what Strip > Add writes (`plans/pes_savefile/model.md` "Kit
  slots and the strip table"); the tables carry `KitSlot*` rows for PES 17 alone. Add the rows
  for the other versions when the Save editor needs kit bindings. (The PES 21 team colours,
  this issue's other half, were measured in Edit mode on 2026-10-13 at the tables' bits.)
- resolved 2026-10-13 (in-game; `spec/pes_savefile/model.md` "The player record") —
  unmodeled player-record bits that real saves set: PES 18 bits 252-254 and PES 16 bits 222
  and 362 are moved by no Edit setting and kept through load, edit and save: retained,
  meaning unknown, not editable. The "most likely `Star`" guess has no support.
- resolved 2026-10-15 (Phase 5 converge: `PlayerEntry::appearance_id` models the field,
  strip-and-seed sets it to -1) — `pes_savefile` player id versus appearance id (2.20i reviewer): the appearance
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
  survey (`.tmp/4_0/apptest`, `fpc` subcommand). Settle with the FPC toggle, built with the
  Save editor's view in Phase 8 (DECISIONS 2026-10-15 "the session wiring only the view
  drives").
- **Texport write is unverified in-game** (2.17h): `Texport::new` synthesizes 18-21 files from
  measured templates and `to_bytes` rewrites read files; both round-trip byte-identical, but no
  generated file has been imported by the game yet (`plans/pes_savefile/verification.md` "Texport write": manual,
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
  FoxDen session 3 (2026-10-14, PES 2019 on the stock 1.6.0.0 exe, scratch
  `.tmp/foxden/plan.md`): (5) FoxDen's own build runs proxy-only on every PES 2019 exe here:
  `host_is_game()` wants a static `dinput8.dll` import, which neither exe has, on disk or in
  the running image (the game loads DirectInput at run time); (6) with that check relaxed
  (test copy), the game dies right after the livecpk Lua state is created, on the stock exe as
  on the patched one: `lua_tolstring` resolves to a misaligned address (RVA 0x4bbba5f, the
  pattern's second pass), most likely mid-function. FoxDen session 4 (2026-10-20, its new
  version on the same stock exe, scratch `.tmp/foxden/plan.md`): (5) and (6) are fixed (the
  host check reads the version resource; `lua_tolstring` resolves aligned), the game runs
  with every script loaded, and the kit-swapper script works on a Studio CPK: kit 1 in Edit
  mode and kit 2 in a match each load the per-kit face package (`<id>p<K>`) or the `kitK`
  texture, falling back to `kitN` when neither exists. Not yet checked: the team as away,
  boots and gloves packages, `dummy_kit`, a `kitN/` folder set.
- open — a run that does not end cleanly leaves its staging (3.z): closing the window while
  the GUI's compile runs ends the process mid-run, and the coordinator's `Cancelled` path does
  not discard the staging either, so `output/.staging/<run>/` can be left beside the previous
  CPK (which stays intact). Fixed with cancellation (`spec/core/gui.md` "Cancellation", Phase 8): the
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
- open (2026-10-09, 4.y-fix4) — the deep pass answers "does this link feed the player's own
  package" with an empty hand-weighted set (`deep/mod.rs` `FaceUse::of_player`), because it
  is the pass that finds the hand-weighted models, while planning and validation answer
  with the real set. For a Fox player whose gloves link combines only through his split
  hands, the linked folder's textures do not count for his `.mtl` paths there, so a
  `mtl_texture_not_found` Warning may be spurious (read, not run); nothing is dropped.
  `FaceUse::combined`'s doc comment ("Empty for a team player ...") was already untrue on
  Fox. Fix: find the hand-weighted models before the face use is built, or build the face
  use after them.
- open (2026-10-11, 5.0e review S3) — a disallowed file a lenient run keeps in a `Common/`
  subfolder no link reaches (`Common/sub/x.doc`) is reported twice on Fox:
  `common_file_disallowed` and `file_not_used` (`validation.rs`, `read_by_no_task`'s
  `!is_read_common_file` arm), where `messages.md` `file_not_used` says a file the
  structure pass already named is not reported (read, not run). Fix: skip a file the
  structure pass reported.
- done (2026-10-13; opened 2026-10-12, the maintainer) — the ATF bot's author forked Rigdio
  (v2.2.1: a local API and event clips) and rebuilt SEN:P-AI (v1.3.0: PES 2019 and 2021
  support, events sent to Rigdio's API); both are cloned in `C:/Data/4cc/Tools_Marco/`.
  Surveyed 2026-10-13: the rulings are in the music player and match tracker plans and
  DECISIONS (the fork survey's rulings), three questions in QUESTIONS; the plans name
  upstream and the forks apart, and the version scope is decided (PES 2019 first, then PES
  2021, then the others).
- resolved 2026-10-13 (in-game, PES 18; `aesthetics_export/settings_toml.md`, the paragraph
  "What a 0 draws was measured on PES 18") — what a hair list at 0 draws: a 0 in `length` or
  `hairline` draws a per-face fallback hair, not a look of its own; a 0 in a list shaven
  greys out changes nothing; changing the length writes 1 into every list at 0. The keys keep
  taking 0.
- resolved 2026-10-14 (5.4a6; `studio_core::pes_process::running_pes`) — no "PES is running" probe exists; `savefile_skipped_pes_running`
  is plan-only. The Save editor's writing commands cannot refuse while the game runs
  (SE-CLI-02) until one lands (`studio_core`'s running-PES poll, `CONTRIBUTING.md`'s `unsafe`
  list).
- resolved 2026-10-14 (5.4a5; `PipelineEvent::Output`) — a tool has no way to print a whole document to stdout: `Message`
  lines are prefixed and folded by the console printer, `println!` is the binary's alone.
  `diff --json` (SE-DIF-02 says stdout) waits on a `studio_core` output event the printer
  writes verbatim.
- resolved 2026-10-14 (5.0m3; the plan keeps him refused: 183 is past every version's top)
  — one PES 16 fixture cup player holds
  `goal_celebration_2 = 183`: PES 16's editor shows "122" (its top) for him and keeps the 183
  through a save of another row; what a match draws for 183 is unmeasured. `from_player`
  still refuses him (`OutOfRange`); `every_fixture_player_round_trips_through_his_settings`
  lists him.
- open (2026-10-15, step 20.0c) — a `stat_bonus` on a stat a tier already targets at 99 gives
  an exact target no player can reach (99 is the game's top), and nothing says so: the first
  Clouds template's "Medals at giga: +2 to every stat" made every gold 101, so `check-aatf`
  placed no player in gold (a TierCount violation on every VGL26 team) instead of naming the
  rule. `aatf_rules.md` is silent. Settle with the next interpreter step: clamp a raised target
  at 99 (the bonus then means nothing for that tier), or refuse it on load where a tier's
  value plus a special's bonus passes 99 (a special's condition may never meet that tier, so
  a load rule could refuse a playable file). The same logic version change fixes the Phase 5
  converge audit's finding (`.tmp/5_6/audit_shapes.md` row 1.7): an explicit `()` on an
  optional `Condition` or `Effects` key (`captain: ()`, `fielded_only_at: ()`, `forbid: ()`)
  reads as `None` in Rust but as present in the logic, which tests the key's presence; the
  logic's `num`/`list`/`table`/`flag` helpers already treat `()` as absent. Only a hand-written
  file reaches it.

---

## Log

The dated log is `docs/LOG.md`.
