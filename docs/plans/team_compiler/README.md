# 4cc Studio — Team compiler plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Data flow

What exists of this is in `spec/team_compiler/README.md` "Data flow".

These lines extend the spec's diagram. In the coordinator's box, `bins_update` also builds the Fox
player rows:

```
│  ─ bins_update (team colors, uniparam; Fox:  │
│    player rows from resolved settings.toml)  │
```

After the writer's box, one more stage:

```
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  pes_savefile: Aesthetics patch + savefile   │
│  ─ Resolve the patch from committed outputs  │
│  ─ Write aesthetics_patch.toml to output/    │
│    (Fox: motions, names; the rest in bins/)  │
│  ─ Local save configured: decrypt, apply the │
│    patch, re-encrypt, save (.bak backup)     │
└──────────────────────────────────────────────┘
```

---

## Crate layout

What exists of this is in `spec/team_compiler/README.md` "Crate layout".

These entries extend the spec's tree of `crates/tools/team_compiler/src/`; `check/` takes the place
of the spec's `check.rs`:

```
├── check/            # live validation orchestration (the lib validates; this schedules)
│   ├── mod.rs        #   two-tier check (shallow archive / deep folder), results cache
│   └── watcher.rs    #   notify + debounce → rescan via reader, recheck requests (desktop only)
├── output/
│   └── savefile.rs   #   builds the aesthetics patch from commit outcomes; applies it to the local save (via pes_savefile)
└── view/
    ├── grid.rs       #   team grid on studio_core's grid widget; ID cell editing
    └── toolbar.rs    #   split Compile button, writability badge, run strip
```

Placement rules:

- `check/` only schedules and caches the export validations of `libs/aesthetics_export` for the
  live grid.

---

## Acceptance

What exists of this is in `spec/team_compiler/README.md` "Acceptance".

**Phase 5 scope.** On Fox, `compile` builds every compiled player's `PlayerAppearance.bin` row
from his `settings.toml` and folder markers, writes the boots and gloves rows of every compiled
player (the default-ID rows included), and writes the aesthetics patch (motions, and a `write_names`
export's names) to the output folder, applying it to the local savefile. A compiled player is
every player a kept player folder of a team export maps to ("Player settings in exports" in the Aesthetics export
plan). Phase 5 withdraws TC-BIN-10, TC-BIN-17 and TC-BIN-22, whose player tables it changes,
and TC-BIN-25 to TC-BIN-27 restate them. Not in Phase 5, so not here: the pre-Fox patch and its
savefile writes (deferred with pre-Fox's other gaps: a compile for PES 15-17 writes no patch and
leaves the savefile byte-identical) and a referee export's settings (worklog issue). PES 18
and 19 appearance rows are built like PES 20 and 21's (`spec/pes_savefile/model.md`, "The Fox
database row"). Test data as in Phase 4. A scenario naming no coverage tag compiles a `Full`
export (a `Midcup` one keeps a player's installed row where a `Full` one writes the default,
`aesthetics_export/settings_toml.md`). The patch halves of TC-BIN-31 to TC-BIN-33 are proven
with the patch (step 5.3b), their table halves before. Unless a
scenario says otherwise, the PES folder is a deployable fixture install whose DpFileList lists
`4cc_08_bins` (holding the three player tables) and the output CPK, and `savefile_path` names
a stripped fixture save of the compile's version carrying team 714 (a test never uses `auto`,
which would find the developer's own save).

**Player tables**

```
TC-BIN-25  GIVEN an installed BootsList.bin of ten pairs, a GloveList.bin and a
           PlayerAppearance.bin, none of them for a team 714 player, and a /co/ export whose
           slot 05 holds boots.fmdl and a settings.toml
           WHEN it is compiled for PES 21
           THEN the CPK's BootsList.bin holds eleven pairs sorted by player id with (71405, 625)
                among them and the ten unchanged; GloveList.bin gains (71405, 0) and
                PlayerAppearance.bin a row for 71405, every installed row unchanged
TC-BIN-26  GIVEN an installed BootsList.bin holding (71405, 7), (71406, 9) and (71407, 55), and a
           /co/ export whose slot 05 holds boots.fmdl, slot 06 no boots, and slot 07 no boots
           and the fpc_off marker
           WHEN it is compiled for PES 21 as a Full export, then as a Midcup export
           THEN the Full run's BootsList.bin holds (71405, 625), (71406, 0) and (71407, 0), and
                the Midcup run's (71405, 625), (71406, 9) and (71407, 0)
TC-BIN-27  GIVEN no PES install configured, and a /co/ export whose slot 05 holds boots.fmdl
           WHEN it is compiled for PES 21
           THEN the CPK holds k0625's boots and no BootsList.bin, GloveList.bin or
                PlayerAppearance.bin, and player_table_missing names each of the three with 1 row
TC-BIN-28  GIVEN an installed PlayerAppearance.bin with a row for 71405 and one for 71406, a /co/
                export whose slot 05 settings.toml sets one appearance key, and whose players.txt
                maps slot 05's folder to 05 and another folder, holding a face model and no
                settings.toml, to 06 and 08
           WHEN it is compiled for PES 21
           THEN 71405's row is the template's defaults with that key's value, 71406's and
                71408's are the template's defaults, and nothing of either installed row remains
TC-BIN-29  GIVEN a /co/ export whose slot 05 carries fpc_on and its settings.toml sets
                boots_id = 12, slot 06 carries fpc_on and no boots or gloves, and slot 07 holds
                boots.fmdl and sets boots_id = 12
           WHEN it is compiled for PES 21
           THEN BootsList.bin holds (71405, 12), (71406, 55) and (71407, 627), GloveList.bin
                (71405, 11), (71406, 11) and (71407, 0), and settings_model_id_conflict names
                slot 07's boots
TC-BIN-30  GIVEN a /co/ export whose slot 05 carries fpc_on, and whose settings.toml sets a key
           the hide preset sets, to another value, the inners on, undershorts and taping;
           and whose slot 06, with no marker, sets the gloves checkbox on and no gloves
           WHEN it is checked and compiled for PES 21
           THEN both report fpc_strip_conflict, fpc_inners_break_hiding,
                fpc_undershorts_break_hiding and fpc_taping_shows_pieces for slot 05 and
                fpc_gloves_checkbox_winter_gloves for slot 06; 71405's row carries the preset's
                value for the key and the inners, undershorts and taping as authored
TC-BIN-31  GIVEN a /co/ export whose slot 05 settings.toml holds facial_hair_type = 15 (PES 18:
                0 to 12), hunching_dribbling = 5 (PES 18: 1 to 3) and dribbling = 2 (PES 20
                and 21 only), and in-game hair (style = 4, color_red = 40)
           WHEN it is compiled for PES 18, and for PES 21
           THEN the PES 18 run reports settings_value_out_of_range for the first two, 71405's
                row has facial hair type 0, its patch holds hunching_dribbling 1 and no
                dribbling key, no finding names dribbling; both runs' rows carry that hair;
                both exit 0
TC-BIN-32  GIVEN an installed pair (71405, 7), and a /co/ export whose slot 05 holds a
           boots.fmdl whose task fails and a settings.toml setting an appearance key and a
           motion key, and whose slot 06 holds a face model that compiles
           WHEN it is compiled for PES 21
           THEN BootsList.bin keeps (71405, 7), 71405's appearance row carries the key, and
                the patch holds his motion keys
TC-BIN-33  GIVEN an installed PlayerAppearance.bin row for 71405 with no value at its default,
                and a /co/ export whose slot 05 holds a face model and a settings.toml with a key
                of the wrong type
           WHEN it is compiled for PES 21
           THEN settings_toml_invalid is reported, 71405's row is the template's defaults, and
                the patch holds his motions at their defaults
TC-BIN-37  GIVEN multi-CPK mode, an installed 4cc_08_bins.cpk (the bins CPK) holding a
                PlayerAppearance.bin row for 71423 and the pair (71423, 7), and a /co/ export
                with no slot 23 folder whose slot 05 holds boots.fmdl
           WHEN it is compiled for PES 21
           THEN the output bins CPK keeps 71423's row and pair as installed, and holds 71405's
                rows from the export
TC-BIN-38  GIVEN a Midcup /co/ export whose only player folder, slot 05, holds ingame_face and
                a settings.toml setting boots_id = 12 and a facial hair type, and no model
           WHEN it is compiled for PES 21
           THEN the output CPK is written and holds the three player tables, with (71405, 12)
                in BootsList.bin and 71405's appearance row from his settings.toml
```

**Aesthetics patch and savefile**

```
TC-SAV-01  GIVEN a /co/ export whose slot 05 settings.toml sets one motion key and a name, and
                a savefile whose team 714 holds 71405 in roster slot 06 and 71406 in slot 05
           WHEN it is compiled for PES 21
           THEN <output_folder_path>/aesthetics_patch.toml names the CPK and holds team 714's
                slot 05 with every motion key (the defaults for those absent) and no name;
                patch_written names it; in the savefile 71405's motion fields are the patch's
                and 71406's unchanged,
                his name and appearance id unchanged, every other byte unchanged, and the
                previous file is the .bak
TC-SAV-02  GIVEN an export with the `write_names` marker whose slot 05 settings.toml has name = true
           and shirt_name = "SNUFFY", in the folder whose name part is Snuffy, and whose slot 06
           folder, mapped to two players by players.txt, sets a name
           WHEN it is compiled for PES 21
           THEN the patch and the savefile carry 71405's name Snuffy and shirt name SNUFFY, both
                players of slot 06's folder carry its name, and settings_toml_name_shared is
                reported
TC-SAV-03  GIVEN a slot 05 settings.toml whose name is longer than PES 21's name field, in a
           `write_names` export
           WHEN it is compiled
           THEN settings_name_invalid is reported, the patch has no name key for 71405, and
                the savefile keeps his name
TC-SAV-04  GIVEN savefile_path naming a missing file
           WHEN the export is compiled
           THEN the patch is written, savefile_missing is reported, and the exit code is 0
TC-SAV-05  GIVEN a PES 19 savefile for a PES 21 compile, and a root holding the /co/ and /egg/
           exports with a savefile holding team 792 and not 714
           WHEN each is compiled
           THEN the first reports savefile_invalid, leaves the savefile byte-identical and the
                PES folder's previous CPK in place; the second reports savefile_team_missing
                naming 714 and applies 792's entries; the first exits 3 (its deployment rolled
                back), the second 1
TC-SAV-06  GIVEN an output folder holding an aesthetics_patch.toml from an earlier run
           WHEN an export with kits and no player folder is compiled
           THEN no patch is written and the earlier one is now aesthetics_patch.toml.bak
TC-SAV-07  GIVEN a savefile whose 71405 appearance id is set
           WHEN a /co/ export with slot 05 is compiled for PES 21
           THEN savefile_player_not_stripped names 71405, and his appearance id stays set
TC-SAV-08  GIVEN a savefile that cannot be replaced (its .bak path taken by a folder)
           WHEN the export is compiled
           THEN savefile_write_failed is reported, the PES folder holds the CPK it held
                before the run, and the exit code is 3
TC-SAV-09  GIVEN an output folder in which aesthetics_patch.toml cannot be written (the path
           taken by a folder)
           WHEN the export is compiled
           THEN patch_write_failed is reported, the PES folder holds the CPK it held before the
                run, the savefile is byte-identical, and the exit code is 3
TC-SAV-10  manual  GIVEN a running PES of the compile target's version with a savefile present
                   WHEN the export is compiled in sideload mode
                   THEN the patch is written naming no CPK, savefile_skipped_pes_running is
                        reported, the savefile is byte-identical, and the exit code is 0; with only
                        a PES of another version running, the savefile is written
TC-SAV-11  GIVEN the installed copy of the output CPK held open without delete sharing
                (TC-DEP-02's degraded run)
           WHEN the export is compiled
           THEN the CPK and the patch are promoted to <output_folder_path>, the savefile is
                byte-identical, and the exit code is 1
TC-SAV-12  GIVEN a /co/ export whose slot 05 settings.toml sets a motion key, and savefile_path
                naming a PES 17 fixture save
           WHEN it is compiled for PES 17
           THEN no aesthetics_patch.toml is written and the savefile is byte-identical
TC-SAV-13  GIVEN a write_names export with a player whose name is too long for the field and
                two players sharing one settings.toml name
           WHEN it is checked, then compiled with --no-deploy
           THEN check reports the same settings_name_invalid and settings_toml_name_shared
                lines as compile
TC-SAV-14  GIVEN a team export and a referee export (/refs/) compiled together for PES 21
           WHEN they are compiled with --no-deploy
           THEN the patch holds one [[teams]] entry, the team export's, and the exit code is 0
TC-SAV-15  GIVEN a folder at <output_folder_path>/aesthetics_patch.toml and an export with a
                notes.txt
           WHEN it is compiled with --no-deploy
           THEN patch_write_failed is reported (Fatal), no teamnotes.txt is written, and the exit
                code is 3
TC-SAV-16  GIVEN a PES 21 savefile present and an export whose settings.toml sets a motion key
           WHEN it is compiled with --no-deploy, and again in --mode test
           THEN no savefile_ finding is reported, the savefile is byte-identical, and each exit
                code is 0
```

**Shared IDs across a Midcup**

The /co/ block's shared IDs are 644 to 660 (`spec/aesthetics_export/player_folders.md` "Assigns
IDs automatically").

```
TC-BIN-34  GIVEN a Full /co/ export holding Boots/Crocs and Boots/adidas, slot 05 linking Crocs
                and slot 06 adidas (a shared folder no player links takes no ID)
           WHEN it is compiled for PES 21
           THEN the CPK holds 4cc_studio/shared_ids/714.toml whose [boots] names 644 adidas
                and 645 Crocs, and BootsList.bin holds (71405, 645) and (71406, 644)
TC-BIN-35  GIVEN an installed CPK below the output holding TC-BIN-34's record, and a Midcup
                /co/ export holding Boots/Birk and Boots/crocs, slot 05 linking Birk and slot 06
                linking crocs
           WHEN it is compiled for PES 21
           THEN Birk takes 646 and crocs keeps 645: BootsList.bin holds (71405, 646) and
                (71406, 645), the CPK's record names 644 adidas, 645 crocs and 646 Birk, and
                no shared_ids_unrecorded is reported
TC-BIN-36  GIVEN no installed CPK holding a /co/ record, and a Midcup /co/ export holding
                Boots/Birk, slot 05 linking it
           WHEN it is compiled for PES 21
           THEN Birk takes 644, BootsList.bin holds (71405, 644), shared_ids_unrecorded names
                team 714, and the CPK's record names 644 Birk
```

**Shared gloves on PES 15-17**

Every `.mtl` of the output moves to its root (`pipeline.md` "3. Per-model-folder parallel
steps (rayon)"), so TC-MOD-93 restates TC-MOD-74, whose packing it changes; TC-MOD-74 is
marked withdrawn in the commit that rewires its test.

```
TC-MOD-93  GIVEN Gloves/Keeper holding glove_l.model with glove_l.mtl, skin.dds, and
                grip/glove_r.model with grip/glove_r.mtl naming grip.dds and skin.dds, and
                grip/grip.dds, slot 05 linking it
           WHEN the export is compiled for PES 17
           THEN the gloves output holds grip/glove_r.model, grip_glove_r.mtl, grip/grip.dds
                and skin.dds, and no grip/glove_r.mtl; glove.xml lists ./glove_l.model and
                ./grip/glove_r.model with material ./grip_glove_r.mtl; grip_glove_r.mtl's
                paths name ./grip/grip.dds and ./skin.dds
```

**Referees**

```
TC-REF-19  GIVEN a refs export whose Ref A holds a face model and a settings.toml setting the
                gloves checkbox on and no gloves
           WHEN it is checked, and compiled for PES 21
           THEN both report fpc_gloves_checkbox_winter_gloves for Ref A; the CPK holds no
                PlayerAppearance.bin row for him, and no patch is written
```

---

## First-release GPU BC7

Desktop GPU BC7 for PES 19–21 uses `block_compression`'s library backend in the first release.
Auto is the default, with CPU fallback and an explicit CPU mode for reproducible builds. The
[library crates plan](../libs/README.md) owns backend selection, startup, batching, memory, and fallback requirements; the
the reproducibility paragraph in `spec/team_compiler/testing.md` "Testing: parity against Red (adapted from Blue)" records the accepted texture-byte exception. Phase 4
builds the texture step CPU-only and Release 0.1.0 ships that way; the GPU backend is integrated
into it in Phase 16, rather than waiting for 3D preview.

## Future Features

Custom GPU kernels and browser GPU acceleration remain deferred until justified by measurements;
the shared library backend does not have to be replaced merely because it shipped first.

### 3D model preview viewport

A future enhancement to the GUI: when the user clicks a player on the progress grid, a 3D viewport
shows a preview of the model.

Scope note: this stays a quick *glance* ("is this the right model") — for actually inspecting a
player folder there is the [Player aesthetics editor](../player_aesthetics_editor.md), which launches
Blender instead of rendering in-app. When this feature is built, the viewport widget goes into a
shared **`libs/model_viewport`** crate (created then, not before — until this feature lands, no
viewport code exists anywhere): the Player aesthetics editor embeds the same widget as a quick
approximate preview beside its launcher, labeled as a preview with Blender as the accurate view,
which gives the crate its second consumer per the core plan's workspace guardrails.

**Feasibility:** High. egui renders via `wgpu`, which supports custom rendering alongside the
immediate-mode UI. The selected folder's structure descriptor can lazily load and parse its model
under the normal memory budget; the resulting vertex/face data is uploaded as GPU buffers and drawn
in the egui painter's render pass.

**Architecture:**

```
User clicks a player on the grid
        │
        ▼
GUI: submit the folder descriptor + source provider to a cancellable background load
  ─ Acquire a memory permit and materialize the selected FmdlFile/ModelFile off the UI thread
  ─ Read vertices (positions, normals, UVs) from the materialized model
  ─ Read faces (index buffer)
  ─ Read bone data (for optional skeleton overlay)
  ─ Convert texture (DDS/FTEX → raw RGBA via dds_convert)
        │
        ▼
GUI: render with wgpu alongside egui
  ─ Upload vertex/index buffers to GPU
  ─ Create texture from raw RGBA
  ─ Draw mesh in an egui custom painter callback
  ─ Camera: orbit/zoom/pan via egui input
  ─ Optional: skeleton wireframe overlay
```

**Implementation:**

```rust
pub struct ModelPreview {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub bones: Vec<BonePreview>,
    pub texture: Option<TexturePreview>,  // raw RGBA
}
// Loaded on a cancellable background worker under a normal memory permit,
// never on the UI thread; the result is tagged with the export revision so
// a stale preview is discarded.

// Conceptual egui_wgpu pseudocode: the exact constructor/API shape depends on
// the egui version pinned when this future feature is implemented.
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(400.0, 300.0),
    egui::Sense::drag(),
);
let callback_state: Arc<dyn egui_wgpu::CallbackTrait> = Arc::new(ModelPreviewCallback {
    gpu_resources,
    camera,
});
// Wrap callback_state in the pinned version's PaintCallback helper and add it
// to the painter; CallbackTrait owns prepare/finish/render-pass integration.
ui.painter().add(make_paint_callback(rect, callback_state));
```

**Performance:** A 20K-vertex model is ~640KB of vertex data — uploaded to GPU once, rendered at
negligible cost. `wgpu` handles millions of vertices trivially. Texture conversion (one 2K DDS → raw
RGBA via `dds_convert`) takes ~10ms. Only one model is previewed at a time, so memory is not a
concern.

**Limitations:**
- **Bind pose only:** The model is shown in its T-pose. PES applies bone transforms at runtime via
  the skeleton and `face_diff.bin`. Showing a posed model would require implementing the PES
  skinning pipeline. Bind pose is sufficient for "is this the right model, does it look right."
- **Not a perfect PES render:** `wgpu` rendering uses standard shading. PES uses custom shaders
  (fox3ddf_blin, fox3dfw_constant, etc.) with specific alpha/blending behavior. The preview shows
  geometry and base texture accurately, but advanced material effects (anti-blur, UV scrolling,
  transparency) won't match the in-game appearance exactly.
- **One texture at a time:** A face model may have multiple materials (face, hair, oral). The
  preview shows the base texture of the selected material, or cycles through them.
- **WASM compatibility:** `wgpu` works in browsers via WebGPU. The 3D preview is available in both
  the desktop and web builds.

**When to implement:** After the GUI is functional (post-Phase 8). The core architecture doesn't
need to change; it's a `wgpu` rendering pass alongside egui. The object model already has all the
data needed.
