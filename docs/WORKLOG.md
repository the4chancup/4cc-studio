# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 4 (Processing logic) closed 2026-10-12. Phases 1 to 3 done (Phase 3 closed 2026-10-02,
Phase 2 2026-09-30).
**Next:** Phase 5 (Savefile integration): its fixed steps 5.0 to 5.0h and step 5.1 done
(2026-10-12), 5.2a and 5.2b1 landed (2026-10-12; Astra R12.A1 over 5.2b1's reworks after the
Pro slots' Tuesday reset), 5.0i (the plans/spec split) done 2026-10-12, 5.2c1 and 5.2c2
landed (2026-10-12, reworked 2026-10-13), 5.0k done, 5.0l done and 5.0m1 landed (2026-10-13),
5.0j done (its last slice 5.0j7), 5.0m2, 5.2a2, 5.2b2b and 5.4a1-4 landed (2026-10-13), 10.0a-c
(`music_export`, ahead of Phase 10), 10.0d, 5.2c3, 5.2d and Autumn 25/Spring 26 (5.2e) landed
2026-10-14, 20.0a (the coverage table) done 2026-10-14; 5.0m3, 5.2c3-r1, 5.2d2, 5.2d2-r1, 5.0m4 and 5.2e (VGL25-27, VTL11, VTL10), 5.2c-r3, 5.2f, 5.2f-r1, 5.4a5-6 and 5.4b done 2026-10-14; then
5.4c, the queued Astra reviews (Handover); step 20.0b (the Ruleset editor draft) done 2026-10-14
(5.2b2's measurements done 2026-10-13, PES 15 included; the official file sets no team settings) (Acceptance written and itemized 2026-10-12). 2.5b (GPU BC7) is
step 16.x (decision entries 2026-09-21 and 2026-09-28). Release target (2026-09-28): 0.1.0
after Phase 8; phase order 1–6, 8, 0.1.0, 7, 9–16 (`plans/core/development_plan.md` "Releases");
first-class target the Fox version the cup moves to around April 2027 ("Target versions").
**Blocked on:** nothing.

**Working mode (since 2026-10-10).** The lead (Claude, in Claude Code) plans, briefs, reviews
and writes what is correctness-critical; the sidekick, SWE-2 through the Devin CLI, implements;
the cross-family reviews run through the `duck` skill (Astra), the lead running them itself.
When a review stops for quota, it is queued here (commit range, plan sections) and retried
once an hour; work continues past it, but no phase closes with one outstanding.
**Handover queue** (2026-10-14): R12.A1 (2 accepted) and C.A1 (4 accepted, step 5.2c-r3)
ran on the brother's Codex slot on 2026-10-14, both loops ending. Left: the interpreter, `3b4b10e..` the 5.2d2-r1 commit (5.2c3, 5.2d, 5.2e's two data
blocks, the Red CBs fix, 5.2c3-r1, 5.2d2, 5.2d2-r1; `aatf_rules.md` "The ruleset schema", "The interpreter", "The host";
SWE-2 rulings `.tmp/parallel/aatf/rulings_as1.md`, `.tmp/5_2/rulings_as2.md`).
And P.A1, 5.4a5-6 (`git diff 41395af 7abd220 -- crates Cargo.toml`; brief
`.tmp/5_4/duck_brief_P_a1.md`, SWE-2 round S1 in `.tmp/5_4/duck_rulings.md`): both Devin Pro
slots refused it on 2026-10-14 (a quota stop, then an internal error twice), so Codex after the
interpreter; it ran there that evening (2 accepted, fixed in 5.4a5-r1; one SWE-2 round left). The interpreter review (I.A1) ran 14:44: its seven concerns
are not ruled yet. Session handover with both, step 5.4c's first moves and the pitfalls met:
`.tmp/lead/handover_2026-10-14.md`.
Phase 3 closed with its reviews queued (the maintainer's
directive of 2026-10-02); Phase 4's converge ran them with its own as the S1-S20 rounds
(map `.tmp/lead/review_queue.md`, briefs `.tmp/duck_brief_S*.md`, rulings
`.tmp/4_y/duck_rulings.md`).

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
TC-GUI-02 manual: checked 2026-10-02 by the maintainer on Windows, the /egg/ tracer export
compiled from the window, the log showing `egg Tracer: Info export_identified (team=/egg/,
id=792)` then `Run finished: exit code 0`. Linux: not yet checked, no machine with a display.

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

Fixed before the itemization:

- [x] 5.0 Check the latest commits made to the unofficial 4ccEditor fork (`Tools_4cc/4ccEditor-1`,
  origin `AnonymousClouds/4ccEditor`; `pes_savefile/README.md` names it the reference): it is
  under constant development. Fetch, then read every commit after `4a95b7c` (2026-06-19, "Add
  Tactics tab for 16-21. Add ability to import Texports from 15-21"), the local clone's head at
  Phase 2's close → done when: each savefile-relevant
  change (schema fields, write behavior, `.4ccs`/`.4cct`/Texport formats) is reflected in
  `pes_savefile` and its plans or recorded as not applying, before any Phase 5 writing code, and
  the commit reached is in the log.
  Done 2026-10-11 (survey `.tmp/5_0/fork_commits.md`, every branch's commits since `4a95b7c`
  read or classified; 37 of the 145 are new work, the rest the base's pre-squash history and
  the VGL fork's): no savefile schema or encoding change; the save-relevant commits fix the
  editor's own bugs (the `.4ccs` tactics seek, a Texport's master key over the save's, imported
  players not persisted), which `pes_savefile`'s design does not share. Reached: the cup's line
  `Autumn_2026_AATF` at `f66e8b4`; the next check starts from every branch head (`git log --all
  --not f66e8b4 c6838c5 206e510 4f3800a a986376 7ef1c3b 91f1599 b42778d`). `aatf_rules.md`'s
  VGL27 moved to `VGL27.B`, whose checker now matches the rules page on Heading, medals' injury
  resistance, out-of-bracket heights and the quota counts. Not applying: `vgl27-Ruleset-qol`'s
  pooled height rule and its fix-database-on-save option (an unreleased third-party branch, the
  latter against the confirmation dialog `save_editor.md` chose). The medal name colour is an
  issue below.
- [x] 5.0b Prototype the two mutation kinds `cargo-mutants` lacks (after Phase 4's converge, so
  it competes with no review): (1) drop a guard, an `if` whose body returns, `continue`s,
  `break`s or errs, its condition replaced with `false`; (2) remove one element of a literal
  list a `split`, `contains` or `matches!` decides by. S10's review (2026-10-09) found the
  color grammar's leading-`+` guard and tab refusal untested with every mutant of the file
  caught (`CONTRIBUTING.md` "Mutation runs"). Upstream checked 2026-10-09: issue #180 "Delete
  statements" open since 2023 with no milestone, its notes leaning against deleting statements
  that hold a `return`; nothing on literals. Shape: a `syn`-based generator listing one text
  patch per mutant, each run in a copy of the tree with a timeout. The maintainer's yes, 2026-10-12 (DECISIONS): the dev-tool crate `crates/dev/extra_mutants`
  (linked into no release binary), with direct `syn` and `proc-macro2` dependencies; the
  prototype runs locally and sequentially, without `mutants.py`'s remote split. `syn` 3 with `full` and
  `visit`, features the workspace already builds it with, so it compiles nothing new (check:
  a clean build compiles `syn` 3 once): the generator only reads the tree and patches text,
  so no `visit-mut`; not `syn` 2, which leaves the tree as `binrw` and `pyo3` move on. Three
  more kinds, trialed with the two (maintainer, 2026-10-12), none of which `cargo-mutants` 27
  makes (it never edits a literal, drops a statement or removes a method call): (3) a
  normalization dropped, a `.trim*()` or `.to_ascii_lowercase()` removed from a chain or
  `eq_ignore_ascii_case` made `==`; (4) an ordering statement dropped, a `sort*`, `dedup*` or
  `reverse` call; (5) a `true`/`false` literal flipped where it is a call argument or a struct
  field. Not trialed: integer literals ±1 (boundaries are `<`→`<=` already, and offsets and
  sizes would add thousands of mutants the goldens catch), string literals, `..`↔`..=`, one
  side of an `&&`. Slices: 5.0b1 the lister with kinds (1)-(2) and the runner; 5.0b2 kinds
  (3)-(5) and a `--kind` filter. 5.0b1 done 2026-10-12 (SWE-2, first attempt: `scan.rs` and
  `main.rs` of `crates/dev/extra_mutants`, `scripts/extra_mutants.py`, `just extra-mutants`;
  78 mutants listed in `aesthetics_export`, and `color_tools`' first run finds `kit.rs:191`'s
  `total == 0` guard untested). 5.0b2 done 2026-10-12 (SWE-2, first attempt: the three kinds
  in `scan.rs`, `main.rs` skipping a file its parent declares under `#[cfg(test)]`, and
  `--kind`, with per-kind `outcomes_<kinds>.tsv`; `aesthetics_export` lists normalize 41,
  order 5, bool 2). → done when: each kind run over `aesthetics_export` on its
  own, its survivors triaged into real gaps and equivalents, and its own count decides
  whether it joins converge (worth it when at least about half its survivors are real gaps).
  Done 2026-10-12 (DECISIONS; triage `.tmp/5_0b/trial/triage.md`): literal 8 real of 9
  survivors and normalize 17 of 17 join converge; guard 2 of 7, order 1 of 3 and bool (no
  survivor) do not. The real gaps are step 5.0g. Reopened for `bool` and `order` (maintainer,
  2026-10-12: no survivor and 1 of 3 are no verdict): each run over `model_convert`,
  `pes_savefile` (bool) or `pes_model` and `fmdl` (order), and `team_compiler`, then decided
  on the total, its real gaps joining 5.0g. Decided 2026-10-12 (DECISIONS "order mutants join
  converge, bool mutants do not"): `order` 15 real of 23 survivors joins `EXTRA_KINDS`; `bool`
  10 of 29 does not. `team_compiler`'s 8 real gaps got tests (`791fc4e`, SWE-2); the other
  crates' are 5.0g2.
- [x] 5.0b3 `extra_mutants`' own gaps, from the review's mutation run (`mutants-diff d81c9ac`,
  2026-10-12: every `aesthetics_export` and `team_compiler` mutant caught, 35 survivors all in
  this crate, triage `.tmp/5_0e/diff/triage.md`): `item_attrs` keeps only the arms whose item
  holds a body the lister visits (twelve equivalent arms); tests for a guard in an impl method
  and a trait default method, `#[cfg(test)]` impls and traits skipped, `#[cfg(windows)]` and
  `#[cfg(feature)]` items scanned, a non-literal list element kept, offsets after `#![..]`, a
  BOM and a shebang, and the binary over a small crate (paths, test-module files skipped,
  usage error) → verify: `just mutants-diff <5.0b3's base>` leaves no survivor in the crate
  Done 2026-10-12 (SWE-2, first attempt): six of the twelve arms stay, by its contradiction
  (an enum discriminant, an array length and a const generic argument are expressions the
  lister visits; a test pins them), the other six go; a list mixing literal and non-literal
  elements now lists its literal ones; ten tests in `scan.rs`, `tests/cli.rs` running the
  binary. `cargo mutants -p extra_mutants`: 121 caught, 5 unviable, none missed.
- [x] 5.0b4 `bool` joins `EXTRA_KINDS` (maintainer, 2026-10-12: its real gaps were the
  trial's worst, silently wrong files) with `.cargo/extra_mutants.toml`, the extra kinds' list
  of ruled equivalents, matched by path, kind and description (a `bool` description names its
  flag), stale entries reported; seeded with the 19 rulings so far; `fmdl`, `pes_model` and
  `archives` run for `bool` the first time (brief `.tmp/5_0b/brief_5_0b4.md`) → verify:
  `team_compiler`, `model_convert` and `pes_savefile` leave no `bool` survivor and no stale
  entry. Done 2026-10-12 (SWE-2, first attempt, plus one rework: three seed entries also hid
  real, tested flags until a method call's description named its receiver; the lead dropped
  the receiver's 40-char cut and named a call's callee as written, removing three
  survivors). 19 entries. First `bool` runs: `fmdl`'s shared-UV-map flag a real gap (a
  corrupt file; tested), its `unwrap_or(false)` replaced by indexing, `archives`' one
  equivalent; `pes_savefile`'s two new `check_text` flips ride 5.1c2's rework
- [x] 5.0g Tests for 5.0b's real gaps, in two slices:
  - [x] 5.0g1 in `aesthetics_export` unless named: `literal`:
    `conventions/player_folder.rs` `is_gloves` (`GloveR`, `HandL`, `HandR`) and
    `is_explicit_face` (`Oral`), `validate/folders.rs` `is_model_content` (`Skl`, `Mtl`,
    `MaterialsToml`), `parse/mod.rs` the roster skipped on `nested_root_conflict`; `normalize`:
    the case tolerance of `is_logo_texture`, `classify`'s `materials.toml`, `normalize`'s content
    folders, `root_file`, the `portrait` stem, `is_kit_texture_name` (`kit`, `kit_`), the kits
    folder's `all` head, `logo_parts`' `_crop`/`_stretch`/`_fit`, `ref_marker.dds`, and the
    whitespace tolerance of `line_color`, `split_folder_name`, `parse_line` and `parse_roster`'s
    blank-line filter; `guard`: `validate/roster.rs` two lines naming one out-of-range slot (two
    `players_txt_slot_invalid`, no `players_txt_slot_duplicate`), and `color_tools` `kit.rs:191`'s
    `total == 0`; `order`: `validate/links.rs` a kit set's `kit10` member after `kit2`. One test
    may cover several sites (a mixed-case export; a padded `players.txt`) → verify: `just
    extra-mutants aesthetics_export --kind literal --kind normalize` misses only
    `validate/roster.rs:89`'s `source_read_failed` (an I/O failure, not reachable by a test)
    Done 2026-10-12 (SWE-2, first attempt): tests in `validate/tests.rs` and beside each site;
    its contradictions accepted: `color_tools` `kit.rs:191` is equivalent (`total == 0` exactly
    when no bin was filled; a `guard` survivor, a kind not run), `parse/roster.rs:52`'s first
    `.trim()` was redundant (the lead folded it into one), the `kit10` sort is `kitF`'s (no
    `kit10` token exists), and the padded folder name is the no-dash arm's (`Kits/ p2`)
  - [x] 5.0g2 the `order` and `bool` gaps of the model crates and `pes_savefile`
    (`.tmp/5_0b/trial/triage.md`): `order`: `pes_model` and `fmdl` `ops/split/build.rs` a
    fragment's faces and loose sets by decreasing projection on the sort vector and a
    component's vertex sets and faces (`pes_model` only), `ops/vertex_enc.rs` a key's kept
    loops by encoding; `model_convert` `formats/fmdl/export.rs` `quantize_weights`' slots by
    decreasing weight, `materials/to_fox.rs` and `materials/to_prefox.rs` a material's textures
    by role before its samplers are written (a source listing its normal map first); `bool`:
    `model_convert` the written FMDL's `antiblur` and `vertex_loop_preservation`, a converted
    pre-Fox mesh group's `visible`, the pre-Fox environment sampler's `srgb`; `pes_savefile`
    `settings_toml/document.rs` `boots_id`/`gloves_id` bounded by their 0-100 range, not the
    field's width → verify: `just extra-mutants <crate> --kind order` (and `--kind bool` for
    `model_convert` and `pes_savefile`) leaves only the survivors the triage rules equivalent
    or not testable
    Done 2026-10-12 (SWE-2, first attempt; 14 tests, all in test modules): each run leaves only
    the ruled survivors; its contradiction accepted: `export.rs`'s `vertex_loop_preservation:
    false` is equivalent (`encode_model` sets the flag after), and `fmdl` `container.rs:232` is
    confirmed equivalent by a census of the machine's 9,606 `.fmdl` files (none out of order)
- [x] 5.0h `just mutants <crate>` runs the adopted extra kinds and reruns Windows-only survivors
  here (maintainer, 2026-10-12): the whole-crate run first runs `literal` and `normalize`
  (`EXTRA_KINDS` in `scripts/mutants.py`), their survivors joining `missed_all.txt`; at
  collection a remote survivor matching `WINDOWS_ONLY_TESTS` (a test that runs on Windows
  only, such as `team_compiler`'s `deploy.rs` error-kind guards, which survived on bonfire at
  Phase 4's close and were caught here) is rerun on this machine, its outcome replacing the
  remote one. Then CONTRIBUTING "Mutation runs" and AGENTS.md name one command → verify:
  the brief's scratch checks (`.tmp/5_0b/brief_mutants_merge.md`). Done 2026-10-12 (SWE-2,
  first attempt: `run_crate` in `extra_mutants.py`, its survivors workspace-relative and
  tagged with the kind; `mutants.py` `EXTRA_KINDS`, `WINDOWS_ONLY_TESTS`, `recheck_windows`,
  the `extra` row and a `missed_all.txt` on every path; the recheck caught `deploy.rs:180`
  here in its scratch check, and `aesthetics_export`'s 26 extra survivors merged)
- [x] 5.0c Red behavior inventory (maintainer, 2026-10-09; after Phase 4 closes, before Phase 5
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
  Done 2026-10-11 against Red `e12aa01`, in seven module groups (`.tmp/5_0c/<group>.md`, the
  rulings in `.tmp/5_0c/rulings.md`): 442 rows, 25 gaps. The Fox compiler's four became step
  5.0e; the 14 pre-Fox ones are an issue below (deferred until the cup next plays a pre-Fox
  version); the Export upgrader's four are its plan's "Open items"; the three log ones are in
  `spec/core/architecture.md` "Diagnostic logging"; a referee's `hair_parts` textures are a
  `QUESTIONS.md` in-game check. The deviations kept are a decision entry, and the plan passages
  that described Red wrongly were corrected.
- [x] 5.0e The Fox compiler's gaps from the Red inventory (5.0c, `.tmp/5_0c/rulings.md` A):
  - (1) a BC4, BC5 or BC7 portrait for PES 15-18 is encoded to BC3 (`player_folders.md`
    "Portraits", decision entry 2026-10-11) → done when: TC-PRT-05 passes, and a BC5 portrait
    under the legacy header comes out BC3 for PES 17
  - (2) census before relaxing: `texture_not_pow2` (either side, mipped, Fox) and
    `kit_texture_too_big` (any target) are stricter than Red (neither side a power of two; kit
    size on pre-Fox only), resting on `libs/dds_convert.md`'s unmeasured "a mipped
    non-power-of-two texture is invalid on Fox" → done when: every real Fox export's textures
    (DDS and FTEX headers through Everything, archives opened) are tallied for a mipped texture
    with exactly one non-power-of-two side and a kit main texture over 2048 or not a power of
    two; none found: a decision entry records Red's laxer rule and the count; some found: an
    in-game check in `QUESTIONS.md` before any rule changes.
    Census done 2026-10-11 (`.tmp/5_0e/census/census.md`: 12,857 distinct textures in 827
    export roots, 1,652 archives opened): 15 mipped textures with one non-power-of-two side in
    Fox exports, 11 Fox kit main textures at 4096 or 8192, none non-power-of-two. Both rules
    stood until the question "Non-power-of-two mipped textures and kits over 2048 on Fox" was
    answered; the answer's rule change is a step of its own (answered 2026-10-12: 5.0f)
  - (3) an FMDL texture path into the game's kit texture folder
    (`/Assets/pes16/model/character/uniform/texture/`): Red reports an Error and keeps the model
    (`e12aa01`, "Report PES uniform texture paths in FMDLs as errors"); Studio keeps a game path
    unlooked-up (`spec/team_compiler/pipeline.md` "Resolved decisions"). Red's reason (its commit message): the
    modded exe swaps kit textures per match, so a fixed path there cannot work, and the member
    must use the dummy kit or a Common path. Ruled: the Warning `fmdl_texture_kit_path`
    (`messages.md`), the model kept with the path unchanged (a Studio Error would drop the
    folder, which Red's Error does not) → done when: TC-MOD-91 passes
  - (4) a `.txt` in a model folder is a note: `file_not_used` like any file no role reads,
    instead of a disallowed type that drops the folder under the default strict check
    (`spec/aesthetics_export/object_model.md` "File-type allowlist") → done when: TC-MOD-92 passes
  Done 2026-10-11 (SWE-2, first attempt; reviews S1-S4 and Astra A1, `.tmp/5_0e/rulings_*.md`):
  `processing/texture.rs` `portrait` takes the version (an exhaustive `match`), `processing/model.rs`
  keeps a kit-folder path and reports it, `aesthetics_export`'s `is_txt_note` admits a `.txt` whose
  stem is no known kind; `paths.rs`, `messages.rs`, `validation.rs`, `validate/folders.rs`, the
  help, tests in `tests/cli/{textures,models}.rs` and `validate/tests.rs`. TC-PRT-05, TC-MOD-91,
  TC-MOD-92.
- [x] 5.0f Red's texture size rules on Fox (DECISIONS 2026-10-12, `messages.md` `texture_not_pow2`
  and `kit_texture_too_big`): a mipped texture with one power-of-two side passes on PES 18-21,
  and a kit main texture over 2048 or not a power of two is refused on PES 15-17 only, where
  `texture_not_pow2` still stays off a kit's main texture → done when: TC-TEX-03's fourth
  folder and TC-CHK-04's two targets pass
  Done 2026-10-12 (SWE-2, first attempt): `deep/texture.rs` (the Fox rule asks neither side to
  be a power of two), `deep/mod.rs` (a kit's main texture takes `MainKit` on pre-Fox only),
  `messages.rs` docs, the help, tests in `deep/texture.rs` and `tests/cli/textures.rs`.

Itemized 2026-10-12 with the Acceptance (`save_editor.md` "Acceptance", the Phase 5 block of
`spec/team_compiler/README.md` "Acceptance"): libs first, each with its fixtures as its spec, then the
two consumers, the in-game check, converge. Before the first step that writes a save (5.3b, 5.4),
the two 4ccEditor forks are checked again for new commits (`Tools_4cc/4ccEditor-1`,
`Tools_4cc/4ccEditor-VGL`; 5.0's survey).

- [x] 5.1 `pes_savefile`'s Phase 5 model (`plans/pes_savefile/model.md` "Planned for Phase 5"), in
  slices:
  - [x] 5.1a the appearance-block player id modeled on the entry, outside `PlayerAppearance`
    (issue "player id versus appearance id"); height and weight out of the key table;
    `shirt_name`; absent = default; `[appearance.unknown]`'s `face` (DECISIONS 2026-10-12, "the
    ingame-face run's undecoded bits travel between versions") → done when: the completeness
    test names every bit of the block on every version, exactly once. Sub-slices, each a commit:
    - [x] 5.1a1 the PES 17+ appearance id: `PlayerEntry.appearance_id` (`None` on 15/16),
      a generated `AppearanceId` row, kept by transplant and conversion, no comparator row.
      Done 2026-10-12 (SWE-2, first attempt; the lead made a Texport write each player's own
      id, which every game `.ted` holds, instead of the brief's 0, and added its test; the
      generator now reproduces the committed tables from `4ccEditor-1`, its two drifts
      fixed)
    - [x] 5.1a2 height and weight out of the key table, Team TOML player keys. Done
      2026-10-12 (SWE-2, first attempt; `.4ccs` import carries them as player keys)
    - [x] 5.1a3 `shirt_name`; `PlayerSettings::apply` takes the target version and checks both
      names against its fields (the text rule shared with Team TOML). Done 2026-10-12
      (SWE-2, first attempt; the lead aligned the commented template line)
    - [x] 5.1a4 absent = default (a `KeySpec` default column); a key the target version lacks
      is skipped. Done 2026-10-12 (SWE-2, first attempt; it used `player_has` where the
      brief named a check that would have skipped every PES 15/16 appearance key; with
      review R1's two fixes)
    - [x] 5.1a5 the unknown bits: `face` on `AppearanceSettings`, `[appearance.unknown] face`
      parsed and emitted (Team TOML refuses the table until 5.1d removes its hex), the
      per-bit completeness test; one slice, since the model half alone breaks the
      settings text round trip. Done 2026-10-12 (SWE-2, first attempt)
    - [x] 5.1a6 review R2's rework: `settings_toml/tests.rs` split in two, an enum naming
      the document where `emit_appearance`'s bool decides more than the range mode, the
      module doc, absent = default tested on PES 15/16, a multibyte `name` at its byte
      capacity. Done 2026-10-12 (SWE-2, first attempt)
    - [x] 5.1a7 review R3's rework: the completeness test's carriers classified (no
      gameplay field inside the block but the PES 15/16 record's join key), an empty
      `[appearance.unknown]` table and `update_toml` keeping `face` tested, the
      cross-version carry test's run length asserted, `hex_run`'s comment. Done 2026-10-12 (SWE-2, first attempt; with review R4's two
      fixes: a stale doc, a third copy of a test lookup made a helper). Step 5.1a done
      2026-10-12: reviews R1 to R4 (SWE-2 and Astra) closed under five accepted each
  - [x] 5.1b the Fox `PlayerAppearance.bin` row from a `PlayerSettings`, the PES 18 and 19
    offsets measured (the games' own rows against save records) → done when: a fixture save's
    records round-trip through rows on PES 18 to 21. Done 2026-10-12 (the lead measured record
    bytes 128 to 184 on PES 18/19; SWE-2, first attempt: `PlayerSettings::appearance_row`
    shares `apply`'s write half without its range checks, since real cup players hold values
    past them, issue "real cup players past the editor ranges"; all 19,596 fixture players
    round-trip byte-exact)
  - [x] 5.1c the aesthetics patch, Fox half: the type, parse, emit, `apply_patch` by player id
    and its refusals (a pre-Fox patch included) → done when: SE-PAT-01 to 04's outcomes hold at
    the lib level. Sub-slices, each a commit (types: `pes_savefile/operations.md` "Aesthetics patch"):
    - [x] 5.1c1 `AestheticsPatch` and its types, `parse` and `to_toml`, the pre-Fox refusal.
      Done 2026-10-12 (SWE-2, first attempt, plus one rework: `to_toml` made fallible
      instead of writing a value `parse` refuses back; with review R5.S1's test witness)
    - [x] 5.1c2 `apply`: the refusals before any write, absent teams and players skipped,
      the `PatchSummary`. Done 2026-10-12 (SWE-2, first attempt, plus one rework: review
      R5.A1's `to_toml` refusals and a mutation survivor). Step 5.1c done 2026-10-12
  - [x] 5.1d the Team TOML: `stripped`, the appearance table written key by key, `ingame_face`
    gone; a test-support export so a tool crate's CLI tests can write a whole fixture save →
    done when: SE-TOML-01 to 08's outcomes hold at the lib level. Sub-slices, each a commit:
    - [x] 5.1d1 `ingame_face` gone, `[players.NN.appearance.unknown] face` in its place
      (absent = untouched), `stripped` on export. Done 2026-10-12 (SWE-2, first attempt;
      `STRIPPED_APPEARANCE_ID` names the -1 where the crate compared against `u32::MAX`)
    - [x] 5.1d2 SE-TOML-01 to 08 at the lib level (each mapped to a test, the missing ones
      written), and the test-support export of a whole fixture save. Done 2026-10-12 (SWE-2,
      first attempt, with review R6.A1's four `apply` tests; `fixture_save` behind the
      `test-support` feature, `team_compiler`'s dev-dependency its first consumer; one
      rework per review: R7.S1, Team TOML's `face` keeps a longer target run's tail; R7.A1,
      R8.S1 and R9.S1, tests made to fail without their behavior, each written value's
      before-value asserted different). Step 5.1d done 2026-10-12
- [x] 5.0i The plans/spec split (maintainer, 2026-10-12; after 5.2b lands, before 5.2c):
  `docs/spec/` holds what the suite does, `docs/plans/` only future work. New plan text is
  written in the present tense (a plan's location says it is future); existing future-tense text
  converts when a section is next edited or moves, never in bulk. Every plan file opens with a
  one-line banner saying nothing in it exists yet and what exists is in `docs/spec/` (a model
  reading an excerpt without its path would take present-tense plan text as fact); briefs and
  reviewer prompts quote plan text with its path. Spec mirrors the plans' layout, headings
  verbatim; a section partly delivered keeps its heading in both trees, the plan's part pointing
  at the spec section it builds on. Phases 1 to 4's delivered sections move now; Phase 5's
  in-progress sections (AATF included) move at Phase 5's close; from then on a phase's close
  moves its sections and corrects what was built differently. A gate script (the lead's: a lint)
  checks that every `docs/plans/` and `docs/spec/` "Section" pointer resolves (AGENTS, GLOSSARY,
  CONTRIBUTING, the worklog, `//!` docs), a heading present in both trees included;
  `DECISIONS.md` stays append-only with a note on the move. Edits: `AGENTS.md` (document table,
  close procedure), `plans/README.md` "How these documents evolve", `GLOSSARY.md` ("Plan"
  rewritten, "Spec" added), the worklog's close-step text, decision entries superseding
  2026-09-10 (4) and 2026-09-19's single home → done when: no Phase 1 to 4 delivered section
  left in `plans/`, every plan file bannered, the pointer check green in `just gates`.
  Done 2026-10-12: 40 documents have a spec copy (`docs/spec/README.md` the index), 12 of
  them whole; eight agents classified every heading, six moved each mixed section's unbuilt
  parts into its plan copy in whole sentences, the lead reviewed each diff. `just
  doc-pointers` also checks relative links and fails on code pointing at a plan outside the
  open phase's files (`OPEN_PHASE_PLANS`, emptied at Phase 5's close); the acceptance
  scanner reads both trees (421 scenarios, 366 proven, as before). The classification found
  spec text describing what was never built, which 5.0j corrects
- [x] 5.0j The spec audited against the code (found by 5.0i's classification; Phases 1 to 4's
  converge audits checked the code against the plan's requirements, not the plan's
  descriptions against the code): each `docs/spec/` file read beside its crates, and every
  passage describing something built differently, or never built and not planned, corrected to
  what exists (an unbuilt idea worth keeping moves to the plan). Known so far:
  - crate-layout trees: `team_compiler` (`plan/manifest.rs`, `plan/refs.rs`, `tests/pipeline.rs`
    absent; `compile.rs`, `deep/`, `validation.rs`, `upgrade.rs`, `events.rs`, `gui_run.rs` and
    others missing), `pes_savefile` (`aesthetics_patch/` a folder, `fixtures.rs`,
    `settings_toml/appearance_row.rs`), `aesthetics_export` (`conventions/kits.rs` is
    `kit_token.rs`, no `validate/structure.rs`, `validate/root.rs`, `validate/kits.rs` and
    `testing.rs` unlisted), `core/architecture.md` "Crate structure"
  - `aesthetics_export/object_model.md` "Model files", "Model folders" (moved to the plan at
    5.0j3) and the end of "Player folders": `ModelFile`, `RawFile`, `FaceModelFolder`, `ModelPart`, `categorize_player`,
    `ExportSlot` were never built (models are planned in `team_compiler`'s `plan/roles.rs`);
    "Core types" (`SharedModelFolder` has `path` and `folder_name`); "Design constraints"
  - `spec/team_compiler/pipeline.md` "Task-batch commit protocol", still an open question, is
    answered by `output/writer.rs` (`commit_folder`, `TaskBatch`)
  - `spec/pes_savefile/container.md` on which count governs (settled: players at 0x34, appearances at
    0x36); `operations.md` the ingame-face hex (gone in 5.1d1)
  - `libs/dds_convert.md` desktop `wgpu` (not enabled: `block_compression` with `bc15`/`bc7`);
    `libs/format_crates.md` the bindings exposing `ops/` (the wheel has none);
    `spec/libs/pipeline.md` "Memory cap" (`FALLBACK_AVAILABLE` is private)
  - `core/README.md` the `phf` row (no such dependency; `paths.rs` uses `match`) and the `sha2`
    row's purpose; `core/architecture.md` launch modes and commands (`--gui` autorun, console
    attachment, `studio.log`, `help`/`update`/`--version` are not built: plan text)
  - `team_compiler/testing.md` "deterministic output across fresh processes with randomized
    hash seeds" (no such test found)
  Sliced per folder, each a SWE-2 brief with the file and its crates; the lead reads every
  diff → done when: every spec file has been read against its code once, each change landed
  - [x] 5.0j1 `docs/spec/core/` (dependency table, crate tree, launch modes, gates, distribution,
    status bar). Done 2026-10-12 (SWE-2, first attempt; the lead deleted the `phf` row instead
    of moving it to the plan, since nothing plans it, and fixed `team_compiler/README.md`'s
    `paths.rs` line)
  - [x] 5.0j2 `docs/spec/libs/` and the format crates (`dds_convert` without `wgpu`, the wheel's
    read/write-only surface, `FALLBACK_AVAILABLE` private, unbuilt consumers moved to plan
    copies). Done 2026-10-12 (SWE-2, first attempt, no lead changes)
  - [x] 5.0j3 `docs/spec/aesthetics_export/` and `kit_config_editor.md` (the unbuilt model
    types and `categorize_player`, the `autopilot` marker and `allocation_scheme_version` moved
    to the plan; the crate tree and "Design constraints" corrected). Done 2026-10-13 (SWE-2,
    first attempt; the lead repointed this list's two pointers). `team_compiler/pipeline.md`
    repeats the `allocation_scheme_version` claim, for its slice
  - [x] 5.0j4 `docs/spec/model_conversion/` and `model_format.md` (the glTF paths marked
    Phase 7, the placement rules and encoder orders as built, the SKL emission rule, FMDL's
    stored texture names, tolerances and counts; `ExportedFox`'s doc comment fixed with the
    spec). Done 2026-10-13 (SWE-2, first attempt; the lead kept the role table's
    `material_texture_not_found`, the plan-only `materials.toml` contract)
  - [x] 5.0j5 `docs/spec/pes_savefile/` (the code was ahead of both trees: the appearance id,
    `height`/`weight`/`stripped`, `[appearance.unknown]` `face`, absent = default, the
    `PlayerAppearance.bin` row and the aesthetics patch's Fox half moved to the spec; the crate
    tree, serial lengths, PES 15's counts and text lengths corrected). Done 2026-10-13 (SWE-2,
    first attempt). The two stale code doc comments it flagged (`TeamSection`'s colours,
    `player_keys.rs`'s `ingame_face`) were fixed with 5.2b2a
  - [x] 5.0j6 `docs/spec/team_compiler/README.md` (the crate tree rewritten to the files that
    exist, the data-flow boxes, the placement rules and the GUI's one action as built,
    `ModelFolder`). Done 2026-10-13 (SWE-2, first attempt)
  - [x] 5.0j7 the rest of `team_compiler` (`pipeline.md`, `settings.md`, `messages.md`,
    `blue_port.md`, `testing.md`). Known: `spec/team_compiler/pipeline.md` "1. Reader" step 4 says contents load
    into a `VirtualTree` (the reader returns owned bytes through `ContentSource`), and its
    `allocation_scheme_version` claim; `tests/parity.rs` and `parity_prefox.rs` point their
    docs at `plans/team_compiler/testing.md`. Done 2026-10-13 (Fable, one rework): the five
    spec files describe the CLI as built, the GUI, savefile and Refs arranger clauses moved to
    their plans, the parity tests' docs point at `spec/team_compiler/testing.md`
- [x] 5.0k `pes_savefile`'s PES 20/21 advanced-instruction table (found 2026-10-12, in-game on
  PES 21, `.tmp/5_2b2/findings.md`): the game stores Anchoring at 0x0A, with 18/19's entries from
  Swarm the Box on shifted up one (Swarm the Box 0x0B, Deep Defensive Line 0x0C, Gegenpress 0x0D,
  Tight Marking 0x0E, Counter Target 0x0F, Wing Back 0x10); `STORED_20` appends Anchoring at 0x10
  instead, so Team TOML reads and writes those six under wrong names. Measured: Anchoring 10,
  False Winger 9, Swarm the Box 11, Deep Defensive Line 12, Counter Target 15 (the measured copy
  is `.tmp/5_2b2/pes21_after/EDIT00000000`, team 701 preset 1). A census of every 17-21 save on
  disk by side (`.tmp/5_2b2/findings.md`, its instruction values by side) fits PES 17's and
  18/19's tables and only the corrected 20/21 one: attack slots hold attack instructions, and
  Tight Marking (set only in a match) is the one defence value never stored. The spec's
  "Version quirks" text is corrected with the code → done
  when: a test decodes the measured values to the in-game names, and the Team TOML round trips
  of the PES 20/21 fixtures still pass. Done 2026-10-12 (SWE-2, first attempt): `STORED_20`
  corrected; tests pin the measured values, the 20/21 table both ways, and every fixture
  instruction on its side (`schema/instruction_golden.rs`); spec `model.md`, `operations.md`
- [x] 5.0l The ingame-face run decoded in-game (lead; `plans/pes_savefile/model.md` "The
  ingame-face run, decoded"): PES 21's whole run measured one setting per player, then spot
  checks on 15 to 20 → done when: every bit of the run is a named field or measured unused, each
  named field a `settings.toml` key. Done 2026-10-13:
  - PES 21 and PES 15 were measured whole, with render checks on 17 and 19, and censuses of the
    games' tables (17 to 21) and the cup saves (15 to 21).
  - The plan names every bit: the field map, two layout families, and PES 15's caps.
  - The key table, ranges and defaults are in `settings_toml.md`, the conversion in
    `operations.md`, and five decision entries are logged.
  - The code lands in 5.0m. Every live save and the faces CPKs were restored by hash.

  Scratch `.tmp/5_0l/` (evidence: `findings.md`).
- [ ] 5.0m The decoded run in `pes_savefile`, before 5.3a (TC-BIN-31 authors hair keys). Plan:
  `plans/pes_savefile/model.md` "The ingame-face run, decoded";
  `plans/aesthetics_export/settings_toml.md`, the template and the ingame-face run's rules;
  `plans/pes_savefile/operations.md`, "Cross-version player conversion" and "Team TOML". Slices,
  each a commit:
  - [ ] 5.0m1 the fields and the keys. One slice, because the completeness test ties every new
    field to its key; most of the diff is table rows. Brief `.tmp/5_0m/brief_5_0m1.md`:
    - every field row with its layout, and the measured-unused table;
    - the nose accessors, and PES 15's `face_type_cap` column;
    - every field a `SettingKey` with the plan's range, comment and default;
    - `[appearance.unknown]` gone from `settings.toml` and Team TOML;
    - `apply` writing the whole run, the unused bits at their values.

    → done when: each version's run is covered bit for bit by one field or unused row, the
    template equals the plan's block, and `from_player` then `apply` gives back every fixture
    run except its unused bits. Done 2026-10-13 (SWE-2, first attempt, two reworks; the lead's
    generated schema rows and plan-literal tests):
    - 92 fields in two layouts, the measured-unused rows, the nose accessors, PES 15's caps;
    - 130 `settings.toml` keys from the plan's table, `[appearance.unknown]` gone;
    - a settings apply writes the unused bits, a Team TOML import leaves them;
    - reviews: SWE-2 S1 to S4, Astra A1 (rulings `.tmp/5_0m/rulings.md`).
  - [x] 5.0m2 the conversion between the two layouts field by field, `FaceRunTruncated` gone
    → done when: a PES 15 → 21 conversion of a cup player keeps his face fields and leaves the
    target's hair bits, and a 21 → 15 one the reverse. Done 2026-10-13 (Fable, first attempt):
    `carry_face` copies the run whole within a layout and field by field between them, a
    non-zero PES 15 nose angle noted as `NoseAngleDropped`; tested on fixture player 70101
    both ways; spec `operations.md`, `model.md`. The plan paragraph "The run between PES 15
    and 16+ converts field by field" is built: it moves at the phase close
  - [x] 5.0m3 the motion ranges per version as measured (`aesthetics_export/settings_toml.md`
    "The motion ranges were measured", the table measured 2026-10-13 in every version's
    editor, `.tmp/ingame_motions/findings.md`): one per-version table of motion tops in
    `pes_savefile`, read by the keys' widest ranges and template comments and by the
    cross-version conversion's cap at the target's top (`pes_savefile/operations.md`
    "Cross-version player conversion"); PES 19's own face-cap column (cheek 5, forehead 6,
    the plan's caps table). The PES 16 player at 183 stays refused by `from_player` (past the
    widest range) → done when: each version's tops equal the plan's table, a value at each
    version's top round-trips through `from_player` then `apply`, a PES 20 player at the tops
    converted to PES 15 holds PES 15's tops, and a PES 19 cheek 5 reads and converts to PES 18
    as 0. Done 2026-10-14 (SWE-2, first
    attempt): `schema::limits::motion_top`, the keys' widest ranges
    and comments read from it, `ConvertNote::MotionCapped`, PES 19's face-cap column. The
    brief's PES 18 half of the cheek test passed before the change (PES 18's cap was 3 both
    ways); the PES 19 target half is the red one. Every fixture player round-trips as before.
    Left: `aesthetics_patch/tests.rs`'s `shifted` repeats the widest ranges as test moduli,
    and `aatf/sample/team.toml` carries the old celebration comments (regenerated at 5.0m4)
  - [x] 5.0m4 the playable ratings on the save's scale and order: `labels::RATINGS` is
    `none 0, C 1, B 2, A 3` and `playable` is read as CF 0 to GK 12 reversed, but every real
    team stores A as 2 (no save stores a 3) in the order CF, SS, LWF, RWF, AMF, DMF, CMF,
    LMF, RMF, CB, LB, RB, GK (a census of 11 799 rated fixture players: 11 795 store 2 at
    their registered position under that order, 5 683 under the reversal; review AS1
    contradictions 1-2). The Team TOML's `playable` labels, `player.rs`'s docs and
    `aatf`'s sample team follow; then `aatf`'s stopgaps go (`engine.rs` `RATINGS`, `team.rs`
    `PLAYABLE_INDEX`), and `PlayerStats` gains the by-index ability setter `aatf`'s
    `values.rs` `write_ability` stands in for → done when: every fixture player's registered
    position reads A, the sample team regenerated reads as before in `aatf`, and the
    stopgaps are gone. Done 2026-10-14 (SWE-2, resumed once
    after a denied delete; decision 2026-10-14): `labels::RATINGS` C, B, A, "3",
    `labels::PLAYABLE_POSITIONS`, `PlayerStats::set_ability`; the census test counts a player
    with any nonzero rating as rated (11 799) and names the four measured exceptions (PES 18
    70101, 78908, 82807; PES 19 84123); the sample's 23 `playable` lines relabelled, its
    bytes and `aatf`'s ratings identical before and after
- [ ] 5.2 `libs/aatf` (`aatf_rules.md` "Development phase and verification"; issue "`apply_tier`
  cannot make a VTL11 medal legal"), sliced per that section → done when: its fixtures
  pass. Slices, each a commit:
  - [x] 5.2a the crate, `Ruleset` and its types (the plan's block), `RULESET` read from a
    data block, the load-time rules with their key paths; the official data block loads →
    done when: one failing file per load-time rule
    reports its rule and key path. Done 2026-10-12
    (SWE-2, first attempt, plus one rework: the lead's review and 35 mutation survivors;
    `ABILITY_STATS` in `pes_savefile`'s now-public `labels` holds the 25 ability stats; `rhai`
    without default features, whose RNG does not build for wasm32)
  - [x] 5.2a2 a serde schema error's key path (`aatf_rules.md` "Loading and validation":
    `serde_path_to_error` wraps the deserializer, the dependency the maintainer approved
    2026-10-13) → done when: a data block with a wrong-typed `tiers[2].free.tricks` reports
    that path. Done 2026-10-13 (Fable, first attempt): the path as the crate prints it
    (`tiers[2].free.tricks`; a missing top-level key's parent is the root, `.`)
  - [ ] 5.2b the host's view of a team (`aatf_rules.md` "The host"). Sub-slices, each a commit:
    - [x] 5.2b1 `TeamView` and `PlayerView` from `pes_savefile`'s model, the tactics answers
      (`fielded_only_at`, `fielded_positions`, formation shapes) in Rust, the engine with every
      accessor of the plan's table but the team settings, `Ruleset::from_data_block` on it
      (`Position` is the registered-position index, GK 0 to CF 12, while
      `PlayerPositions::playable` runs CF 0 to GK 12, so a rating lookup converts) → done when:
      each accessor is tested on fixture teams, through a script. Done 2026-10-12 (SWE-2, first
      attempt; `pes_savefile` gains `PlayerStats::abilities` and `names::name_color`; `print`
      runs on the shared engine and writes nowhere)
    - [ ] 5.2b2 the team settings' accessors: the man-marking assignments in `pes_savefile`'s
      tactics model (PES 15 to 21, offsets in `aatf_rules.md` "The host"), the auto flags (off
      is 0; PES 15's three at +0x1FE to +0x200, not modeled yet) → done when: each setting reads
      right on a fixture team set both ways. Measured 2026-10-12/13 (in-game on PES 15, 16 and
      21, and a census of every 15 to 21 save on disk; `.tmp/5_2b2/findings.md`): man marking is
      per-preset assignment bytes, not an advanced instruction, its values the opponent's
      formation slots (read back in-game on PES 15 and, 2026-10-13, PES 21); an advanced
      instruction's player byte is a roster slot, as `AdvancedInstruction::player` says (read
      back on PES 21, a team whose eleven is not in roster order). Sub-slices, each a commit:
      - [x] 5.2b2a `pes_savefile` models them: `TacticsPreset::man_marking` on every version,
        PES 15's three auto flags, both entering the schema generator as `MEASURED_FIELDS`
        (`docs/DECISIONS.md` 2026-10-13); Team TOML carries `man_marking` per preset; the aatf
        sample team rewritten with them. Done 2026-10-13 (SWE-2, first attempt; the lead's
        spec text and two stale doc comments the 5.0j5 audit found)
      - [x] 5.2b2b the `aatf` accessors (`team.man_marking`, the auto flags; the plan's table
        rows) → done when: each reads right on a fixture team set both ways. Done 2026-10-13
        (Fable, first attempt): `TeamView::man_marking` beside the other tactics answers, the
        auto values `()` on a version without them, each tested through a script
  - [ ] 5.2c `logic.rhai`'s `check_team` for the official file, `Violation` from its maps,
    `file.rs` (assembling a file, its section split, logic identity), the sample team, the official known-legal and known-violating fixtures, the operation
    limit → done when: each special of the official file is exercised by a fixture. Sub-slices,
    each a commit:
    - [x] 5.2c1 `Violation` and its conversion from the map, `Rules` (load steps 1 and 2, the
      official file assembled from its data block and a stub `logic.rhai` whose `check_team`
      returns only the custom checks, `check_team`) → done when: each kind converts from its
      map, each bad map fails with its index, each step-1 rule fails on a file of its own.
      Done 2026-10-12 (SWE-2, first attempt, one small rework; `call_fn` re-runs the top level
      on a fresh scope, so a function reads `global::RULESET`, tested)
    - [x] 5.2c2 `TeamView::from_team_toml`, the sample team (`.tmp/5_2/sample_714.toml` is the
      lead's extraction, `.tmp/5_2/sample_team/` the probe), load step 3, the section split and
      `LOGIC_VERSIONS` → done when: a view from a fixture team's own document equals
      `TeamView::new` on every fixture version, and each identity case is classified.
      Done 2026-10-12 (SWE-2, first attempt; lead fixes: one shared `section_hash`, and the
      version lookup as one `position` match, which removed an equivalent mutant). An import
      note reaches `TeamViewError::TeamToml` as its `Debug` text: `ImportNote` has no `Display`
      yet, which the Team creator's check (Phase 6) will want. Review C1.S1 (SWE-2) over
      5.2c1-c2: 5 of 5 accepted, rework r1 landed 2026-10-13 (set-piece none values, a `let
      RULESET` refused, stale docs, the import-note arm tested, `team_toml`'s key-0 underflow).
      Review C2.S1 (SWE-2): 3 of 5 accepted, rework r2 landed 2026-10-13 (comments, a test
      pinning the last `const RULESET`; the sidekick showed rhai's scope keeps one entry per
      name, so the lookup stayed `find`), which ends the SWE-2 loop; Astra queued (Handover)
    - [~] 5.2c3 the interpreter's `check_team` for the official file, its fixtures, the
      operation limit. Landed 2026-10-14 (side-lead, `b251fd8`, `e2fe165`): every special of
      the official file and each suggestion kind has a fixture; the operation limit is ten
      times the measured 482 556 operations of the official file on the sample team (89.5 ms
      in release); rhai's expression depths pinned (debug and release defaults differ). Found:
      the playable scale is C 0, B 1, A 2 and the order is not a plain reversal (step 5.0m4).
      The side-lead bounded "Red CBs up to 189" to 189; the maintainer ruled for `aatf.cpp`'s
      reading (the 4 cm at every height), applied by the lead with the Red legal fixture's CBs
      at 189. Review AS1 (SWE-2): 6 of 12 accepted (`.tmp/parallel/aatf/rulings_as1.md`)
      - [x] 5.2c3-r1 AS1's rework: `AllowanceWithBracket` looks inside `when.not`; a `choice`
        option carrying `stat_target`, `require_cards`, `forbid`, `height_allowance` or a
        nested `choice` refused at load; `suggestions`' weak foot read through `num`; a Team
        TOML without `[tactics]` leaves no player fielded (0xFF); fixtures for `weight`'s floor
        and `age`'s lower bound; `PositionRange`'s doc → done when: each has its red-first
        test, then a SWE-2 round AS2 over the rework. Done 2026-10-14 (SWE-2, with 5.2d2):
        item 3's crash did not reproduce (`num` already guarded both reads), kept as a
        consistency change
  - [x] 5.2d `tier_values`, `card_limits`, `apply_tier`. Landed 2026-10-14 (side-lead,
    `978a33b`): `apply_tier` then `check_team` reports nothing for every official tier (the
    Green silver giant and the Red manlet included); VTL11's name colour, a `QUESTIONS.md`
    entry, answered 2026-10-14 by a tier field (step 5.2d2)
  - [x] 5.2d2 the schema fields step 20.0a and VTL11's name colour added (`aatf_rules.md` "The
    ruleset schema", "The host", the violation kinds): `Tier::name_color` with its check and
    `apply_tier`'s write; `Condition::ranges`, `strong_foot`, `any`; `Effects::form` and
    `injury_resistance`; the accessors `player.strong_foot` and `team.formation`; briefed
    with 5.2c3-r1 to SWE-2 (`.tmp/5_2/brief_5_2c3_r1_d2.md`), round AS2 reviewing both → done when: each field has a red-first fixture
    (a special that fires through it, and one that does not), `apply_tier` then `check_team`
    reports nothing for a tier with a name colour and for a tier without one under a
    ruleset that sets them (a name with the other tier's code stripped, a name with an
    unrelated code kept), and `team.formation`'s slots match the fixture save's preset 1
    Landed 2026-10-14 (SWE-2, first attempt; lead fix: `Effects::form`'s doc). Review AS2
    (SWE-2): 4 of 6 accepted (`.tmp/5_2/rulings_as2.md`), so the SWE-2 loop ends after its
    rework; then Astra over 5.2c3, 5.2d and 5.2d2 (Handover)
    - [x] 5.2d2-r1 AS2's rework: the `ranges` rows for `weight`, `injury_resistance`,
      `weak_foot_usage`, `weak_foot_accuracy`; one `raised_target` helper for the form and
      injury-resistance targets; the `name_color` row of `bad_proposal_field_values_are_schema`;
      a present `weak_foot` names both halves (plan, decision 2026-10-14) → done when: each
      has its red-first test. Done 2026-10-14 (SWE-2,
      with the five tests `just mutants-diff 5c1f637` asked for in `music_export` and the Save
      editor: rulings `.tmp/lead/mutants_5c1f637_rulings.md`; one equivalent excluded in
      `.cargo/mutants.toml`, `StartOffset::parse`'s dead arm removed)
  - [x] 5.2e the Autumn 25, Spring 26, VGL25, VGL26, VGL27, VTL11 and VTL10 data blocks and their
    fixtures → done when: the plan's per-ruleset fixtures pass. Autumn 25 and Spring 26 landed
    2026-10-14 (side-lead, `0aceb0b`: the sample team, a real team built under Spring 26,
    reports nothing there as written); VGL25 landed 2026-10-14 (SWE-2: `rulesets/vgl25.rhai`,
    `src/fixtures/vgl25.rs`, nine fixtures on the sample team made legal; neither VGL25 nor
    VGL26 has a by-tier `choice` option or a group card freed by name, so review AS1's
    concerns 2 and 3 stay closed); VGL26 landed 2026-10-14 (SWE-2 plus the lead's goalkeeper
    height special: `rulesets/vgl26.rhai`, `src/fixtures/vgl26.rs`, eleven fixtures, the fluid
    ones on the sample built as PES 21 by `Records::sample_on`; review R12.A1's two engine
    tests); VGL27 landed 2026-10-14 (SWE-2 plus the lead's narrowing of its goalkeeper special
    to 185 cm: `rulesets/vgl27.rhai`, `src/fixtures/vgl27.rs`, sixteen fixtures); VTL11 in two
    slices (its single brief overran SWE-2's output limit after the reading): (a) landed
    2026-10-14, `assemble` keeps a `rulesets/` file's CUSTOM CHECKS section, and the card tally
    counts a group's cards only when not free by name (review AS1's concern 3, which VTL11's
    Long Ranger reaches; logic 1's hash edited in place, unreleased); (b) landed 2026-10-14
    (SWE-2, resumed once after a truncation: `rulesets/vtl11.rhai` with its CUSTOM CHECKS
    section, `src/fixtures/vtl11.rs`, fifteen fixtures, the PES 16 team settings on a PES 16
    build of the sample); VTL10 landed 2026-10-14 (SWE-2: `rulesets/vtl10.rhai` from
    VTLEditor `2cf222f`, `src/fixtures/vtl10.rs`, eighteen fixtures; the plan's "VTL10" lists
    its differences from VTL11; the VTL wiki's VTL10 page describes another ruleset, which
    5.2f's parity run on the VTL10 save settles)
  - [x] 5.2c-r3 Astra C.A1's four accepted concerns, done 2026-10-14 (SWE-2): the
    `ViolationKind` target docs read as the plan (`at_most`: under is legal, over is the
    violation); tests for a custom check's two findings in order, a Team TOML roster with an
    interior empty slot, and `Rules::logic()` (official `Current`, edited `Modified`);
    `violation.rs`, `rules.rs`, `team.rs`
  - [x] 5.2f the parity runs, done 2026-10-14 (lead; `.tmp/5_2f/`: each checker built
    headless from its ruleset commit, run three times per save, `findings.md`): Autumn 25 and
    Spring 26 on their two cup saves each, Autumn 26 on all four, VTL10, VTL11 and VGL26 on
    their saves, VGL27 on VGL26's. Legality agrees on every team but VGL26's support-settings
    rule (issue below) and one first-preset finding the page adds; the other differences are
    the plan's "Differences every legacy checker shows" and the per-ruleset ones, except
    VTL11's two playing-style warnings (5.2f-r1); `aatf_rules.md` (Background, "VTL10",
    "Development phase and verification")
  - [x] 5.2f-r1 VTL11's two warnings, done 2026-10-14 (SWE-2): two `severity: "warning"`
    specials in `rulesets/vtl11.rhai`; the legal fixture team's captain becomes a Prolific
    Winger (a Goal Poacher rated A at CF draws the warning); `the_playing_style_warnings_at_cf`.
    Re-run on the VTL11 save: the 22 warnings now match the checker's
  - [ ] 5.2f-r2 Where PES 21 stores "switch preset tactic" (`aatf_rules.md` "VGL26", the
    paragraph on the support settings; in-game, so ask the maintainer before taking focus; back
    up the saves first): toggle it on one team in Edit mode, save, and diff the EDIT file and
    the SYSTEM file against their backups; then toggle the team's other support settings one at
    a time the same way, to name the three bytes `TeamAutoFlags` reads. Then the four rulesets
    (VGL26, VGL27 require it on; VTL10, VTL11 off) keep the rule on the right byte, or drop it
    when the setting is not per team → done when: the byte (or "not in the EDIT file") is
    measured and written in `pes_savefile`'s model plan, and each of the four data blocks
    follows it with its fixtures
- [ ] 5.3 Team compiler savefile writing, Fox, after 5.1:
  - [ ] 5.3a the player tables: appearance rows, default-ID rows, FPC presets and their
    findings, the out-of-range rule (a key's default; a key the target lacks dropped silently;
    `pes_savefile` exposes no per-version range yet, so it gains one first: today
    `PlayerSettings::apply` takes a PES 21 file's `hunching_dribbling = 5` for PES 18 and the
    codec refuses it only at write, review R2.S1-1),
    the `autopilot` marker in `aesthetics_export`; settle issue "shared IDs and retained rows"
    first; TC-BIN-10/17/22 marked `withdrawn: restated as TC-BIN-25/26/27` in the commit that
    rewires their citing tests (`tests/cli/bins.rs`) → done when: TC-BIN-25 to 33 and TC-REF-19
    pass (TC-BIN-31's PES 18 half after 5.1b)
  - [ ] 5.3b the patch and the local savefile → done when: TC-SAV-01 to 12 pass (TC-SAV-10 a
    `manual:` line)
- [ ] 5.4 The Save editor crate (skeleton, settings, egui-free `session/`, CLI), after 5.1:
  - [ ] 5.4a `export-toml`, `import-toml`, `apply-patch`, `transplant`, `diff`,
    `export-teams-list` → done when: SE-CLI, SE-TOML, SE-PAT, SE-TRN, SE-DIF and SE-TL pass.
    Slices, each a commit:
    - [x] 5.4a1 the crate (`crates/tools/save_editor`: `lib.rs`, `settings.rs`, `events.rs`,
      `messages.rs`, `session/`, `cli.rs`, `help/`), registered in `studio` (its window a
      placeholder line until the GUI step), `export-toml`, the in-process CLI sandbox;
      SE-TOML-02, 03. Done 2026-10-13 (Fable side-lead)
    - [x] 5.4a2 `import-toml`, `apply-patch`, the `.bak` write rule; SE-CLI-01, SE-TOML-01,
      04-08, SE-PAT-01-04. Done 2026-10-13 (Fable side-lead)
    - [x] 5.4a3 `transplant`, `diff` (`diff.rs`); SE-TRN-01-03, SE-DIF-01, 03, 04. Done
      2026-10-13 (Fable side-lead)
    - [x] 5.4a4 `export-teams-list` (`teams_list.rs`); SE-TL-01-04 (SE-TL-04 added: the PES 21
      fixture's two `/4ccg/`). Done 2026-10-13 (Fable side-lead; the lead's fix: with no data
      directory the working list is the upstream one, not a `teams_list.txt` in the current
      directory)
    - [x] 5.4a5 `diff --json` (SE-DIF-02) through `PipelineEvent::Output`, done 2026-10-14
      (SWE-2): `studio_core` `events.rs` (`Output`, `EventLines`); `save_editor` `diff.rs`
      builds one `DiffReport` rendered as findings or as JSON, `Report::output`, `--json`;
      `serde_json` 1.0.151; the test helper `Run::stdout`. Rework 5.4a5-r1 (Astra P.A1's two
      findings: a stripped player only in the first save reported twice; `--json` with a scope
      untested), done 2026-10-14 (SWE-2)
    - [x] 5.4a6 the write refusal while PES runs, done 2026-10-14 (SWE-2): `studio_core`
      `pes_process.rs` (`running_pes`: Toolhelp on Windows, `/proc/<pid>/cmdline` on Linux);
      `save_editor` `cli.rs` `refuse_while_running` first in `import-toml`, `apply-patch` and
      `transplant`; help `01_command_line.md`. Checked by the lead with a stand-in (`ping.exe`
      copied as `PES2021.exe`; the probe reads only the process name; `.tmp/5_4/se_cli_02/`):
      `transplant` refused with exit code 2 naming PES 2021 and `--out` unwritten, the same
      command wrote it with no stand-in, before and after; `import-toml` and `apply-patch`
      given a missing file refused before reading it. Narrowed the same day to the PES of the
      save's version (DECISIONS 2026-10-14; stand-in check `.tmp/5_4/se_cli_02/result_version.txt`:
      `PES2021.exe` refuses the PES 21 save, `PES2015.exe` does not). SE-CLI-02's `manual: checked`
      line waits for 5.4c's two commands and a real game
  - [x] 5.4b `check-aatf`, after 5.2 → done when: SE-AATF-01 to 03 pass. Done 2026-10-14
    (SWE-2, first attempt): `cli.rs` `check_aatf`, `parse_teams`; `messages.rs`
    `aatf_violation`, `aatf_check_failed`, `violation_rule`; the `aatf_rules` setting; help;
    `tests/cli/check_aatf.rs` on the lead's fixtures `tests/fixtures/aatf/` (PES 15 team 714
    made legal, and legal but for one warning, under the official ruleset)
  - [ ] 5.4c `strip-and-seed` and `check-stripped` (measure issue "strip-and-seed and a
    base-copied face" first, and settle issue "real cup players past the editor ranges"); both
    call `refuse_while_running` first (`check-stripped` with `--strip-again`) → done when: SE-STR-01 to 06 pass on PES 21, and on PES 18 and 19
    after 5.1b; then SE-CLI-02 checked by hand with a running game, all five commands
- [ ] 5.5 The in-game check (`development_plan.md` Phase 5 "Verification"), by the lead through
  the harness: a stripped and seeded save shows every player as before; a compiled team's rows
  show; a file that sets no face or hair key shows the default face → done when: each a
  `manual: checked` line here
- [ ] 5.6 Converge (`AGENTS.md` "Closing a phase")

### Phase 10 — Music tools

The phase's steps are itemized when Phase 9 closes; its format crate was built ahead of it:

- [~] 10.0 `libs/music_export`: the `.4ccm` format crate (`music_player.md` "The `.4ccm` format")
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
  - [ ] 10.0e the plan's "Model" section extended with `Verdict`, `MatchState`, `PriorityList` as
    built; `just mutants music_export`, `just clef music_export`, each survivor and flag ruled

### Phase 16 — Polish and distribution

Steps are itemized when Phase 15 closes; one is fixed already:

- [ ] 16.x `dds_convert` GPU BC7 (deferred from 2.5b to Phase 4, decision entry 2026-09-21, then
  past Release 0.1.0, decision entry 2026-09-28; spec `libs/README.md` "First-release desktop GPU
  BC7"): `block_compression`'s wgpu backend on a Vulkan/Metal device, CPU fallback with the
  fallback reason reported, cold pipeline creation and upload/readback measured first, then the
  texture step's bounded batches → verify: GPU and CPU outputs decode within the same tolerance
  on the `dds_convert` fixtures; the fallback path exercised by forcing no adapter

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

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history. A question only the maintainer
can answer is not an issue: it goes to `docs/QUESTIONS.md`, and an issue that waits on one
points there.

- open (2026-10-14, 5.2f) — every team of the VGL26 cup save fails VGL26's "Support settings"
  special on `auto_preset_change` (all three PES 21 cup saves hold it off): the rule follows the
  rules page, which speaks of the deadline export. Answered by the maintainer the same day: every
  cup manager keeps "switch preset tactic" on, so the setting is likely not stored per team in the
  EDIT file (the SYSTEM file is the candidate). Waits on step 5.2f-r2's in-game check.
- open (2026-10-11, Phase 5 Acceptance; deferred until the cup next plays a pre-Fox version) —
  the pre-Fox aesthetics patch (`pes_savefile/operations.md` "Aesthetics patch", the pre-Fox
  half; `spec/team_compiler/pipeline.md` "Post-processing"): every appearance field, the resolved boots and gloves
  IDs through commit-conditioned mutations, the edit flags and the base-copy ID (issue below).
  Until then a compile for PES 15-17 writes no patch and leaves the savefile alone (decision
  2026-10-11).
- open (2026-10-11, Phase 5 Acceptance) — a referee export's `settings.toml` reaches no table or
  patch: a referee's look is the template's `RefereeAppearance.bin` (`blue_port.md`), whose rows
  nothing writes, and `blue_port.md` has `fpc_off` apply its settings preset to a referee on
  PES 18-21. Settle with a Fox refkit (the issue on a referee's body under `fpc_off`): which
  table a referee's settings go to, and TC-REF-19's outcome then.
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
- open — shared IDs and retained rows: a team's shared boots/gloves IDs are assigned in
  alphabetical folder order from the export being compiled (`player_folders.md` "Assigns IDs
  automatically"), while a player the export does not compile keeps his installed row
  (`spec/team_compiler/pipeline.md` "Bins accumulation"). A `Midcup` export holding shared folder B but not A
  gives B the ID A had in the full CPK, so A's wearers, their rows untouched, now wear B.
  The same holds for pre-Fox save IDs. Settle before 4.22: a midcup keeps the full export's
  assignment (from where?), or must carry every shared folder its team uses (review S5.A2
  item 1).
- open — generation from a stripped save: `settings_toml.md` has generation skip a player
  at -1, so an old-format export migrated against the cup's stripped Fox save gets no
  `settings.toml` for him and his next compile writes all-default rows over the look the
  tables hold. Reading his installed table rows (the bins walk) instead would keep it.
  Settle with the Export upgrader's generation, Phase 6 (review S5.10 item 3).
- open — the Export upgrader's `fpc_on` detection (`export_upgrader.md` step 8, "settings
  match the enable preset"): the cup's FPC players mostly ride their bodies in per-player
  boots IDs, so their boots field holds that ID, not 55, and Test 1's save had every /a/
  player FPC while `is_fpc_player` found none (`.tmp/4_0/apptest/results.txt`, run 2). Matching
  the strip fields alone would also mark a dressed player with long sleeves, tucked shirt and
  short socks. Settle the fields compared before Phase 6 (review S5.11 item 2).
- open — real cup players past the editor ranges: `PlayerSettings::from_player` refuses a
  stored value its key's kind cannot represent (`OutOfRange`, documented as "a hostile save's
  out-of-range bits"), but the PES 18 fixture save holds 79 such players and the PES 19 one 26
  (`cheek_type` 4 to 5 where the range ends at 3, `forehead_type` 6 where it ends at 5, inside
  the fields' bits; found at 5.1b). So a `settings.toml` cannot be generated for them, and
  strip-and-seed, whose rows must equal each player's record bytes (SE-STR-01), cannot take
  the `from_player` route: `appearance_row` itself writes such a value as is (the field's bits
  are its bound), but the Save editor has no public way to build those settings. Settle before
  5.4c (a row straight from a `PlayerEntry`, or wider ranges if the game shows those values),
  and before any tool generates `settings.toml` from a save.
- open — strip-and-seed and a base-copied face: the `PlayerAppearance.bin` row carries no
  base-copy id (`plans/pes_savefile/model.md`, "The Fox database row"), which holds a face-transplant
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
  target it compiles, so a diff is the author's responsibility, as with Red. The
  investigation waits for the maintainer's go (`docs/QUESTIONS.md` "Face diffs are
  engine-specific").
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
- resolved 2026-10-13 (in-game; `plans/pes_savefile/model.md` "The player record") —
  unmodeled player-record bits that real saves set: PES 18 bits 252-254 and PES 16 bits 222
  and 362 are moved by no Edit setting and kept through load, edit and save: retained,
  meaning unknown, not editable. The "most likely `Star`" guess has no support.
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
  pattern's second pass), most likely mid-function. The kit-swapper script has not run yet.
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

---

## Log

The dated log is `docs/LOG.md`.
