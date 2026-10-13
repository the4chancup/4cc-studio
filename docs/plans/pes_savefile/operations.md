# 4cc Studio — Savefile plan: Operations

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Cross-version player conversion

### What the Rust module is

What exists of this is in `spec/pes_savefile/operations.md` "What the Rust module is".

Not in this module (open): the motion ranges narrow from 20/21 to 19 and earlier (corner kick
1–10 vs 1–6, free kick 1–20 vs 1–16, penalty 1–7 vs 1–4, see the `settings.toml` key table); a
20/21 → 19 conversion copies such a value through uncapped. Whether the game clamps, ignores or
misbehaves on an out-of-range motion is unmeasured; measure before capping.

**The run between PES 15 and 16+ converts field by field (step after 5.0l).** The spec's
`min(len)` prefix copy rests on a premise that turned out wrong: that the run's layout is one
layout on every version. PES 15 stores its hair differently (`model.md` "The ingame-face run,
decoded"), so a prefix copy turns a PES 15 variation id into a 16 to 21 hairstyle and cropped
style, and sets list bits past the shipped models. The hair can vanish, and a set cropped-style
high bit crashes PES 21.

So a conversion whose two versions are in different layout families (PES 15 on one side, 16 to
21 on the other) does this:

- it reads every field both families share from the source;
- it writes those fields onto the target's run;
- it leaves the target's own family-only and unused bits as the template player holds them;
- it re-codes the nose: PES 15's 3 × type + angle becomes the type alone, and back with angle 0.

`FaceRunTruncated` goes, since no byte is copied by position any more. Between two versions of
one family, the run is still copied whole.

The eleven face types are then capped as now, with PES 15's caps measured. They are wider than
16's:

| Face type | PES 15 | PES 16–19 | PES 20–21 |
|---|---|---|---|
| cheek | 14 | 3 | 3 |
| forehead | 6 | 5 | 5 |
| facial hair | 29 | 12 | 19 |
| laughter lines | 6 | 4 | 4 |
| upper eyelid | 14 | 6 | 7 |
| lower eyelid | 14 | 2 | 6 |
| eyebrow | 17 | 5 | 7 |
| neck line | 5 | 2 | 3 |
| nose | 11 | 6 | 7 |
| upper lip | 7 | 3 | 4 |
| lower lip | 7 | 2 | 4 |

The PES 15 column joins the editor's range and the census of the game's faces. Neck line 4 and 5
and lips 7 have no editor entry, but the game draws them as distinct models. PES 17 to 19 stay
assumed equal to 16.

## Interchange formats

### Team TOML (new, replaces `.4ccs`/`.4cct`)

What exists of this is in `spec/pes_savefile/operations.md` "Team TOML (new, replaces `.4ccs`/`.4cct`)".

Open (the step after 5.0l): `[players.NN.appearance.unknown]` leaves with `settings.toml`'s,
since every bit of the ingame-face run is now a key or measured unused (`model.md` "The
ingame-face run, decoded"). The players' appearance tables gain the new face and hair keys.
Import keeps "absent = untouched", so a run's unused bits keep the save's values.

### Aesthetics patch (new)

What exists of this is in `spec/pes_savefile/operations.md` "Aesthetics patch (new)" (the
format, the Fox half's keys and `apply`, `interchange/aesthetics_patch`'s types).

Still open:

- **The pre-Fox half of the format.** On **pre-Fox** (PES 15–17) the patch holds everything
  below, as the only route for aesthetics, names likewise only from an autopilot export: per
  player, the `PlayerSettings` schema plus `name`, `shirt_name` and the compiler-owned
  `edit_flags`, with `boots_id`/`gloves_id` over their full stored range — the resolved ID is
  the one the order in the [Aesthetics export plan](../aesthetics_export/settings_toml.md)'s
  "Player settings in exports" reaches (a committed custom output's assigned ID, else the
  authored stock ID 0 to 100, else the FPC marker's default), and a category whose requested
  output failed carries no key, so the save keeps its current ID — the same union Team TOML's
  player aesthetics section holds; nothing is invented for the patch. Every appearance field is
  written, with `settings.toml`'s defaults for keys its file lacks, so a player's appearance is
  his export's whatever the save held.

  ```toml
  [teams.players.03]           # slot 03 → player 70203; the Fox keys are the spec's
  name = "Snuffy"              # resolved: `name = true` became the string
  shirt_name = "SNUFFY"
  # pre-Fox only from here on
  boots_id = 143               # resolved: the assigned custom ID, the authored stock ID or the marker's default
  gloves_id = 0
  edit_flags = 12

  [teams.players.03.appearance]
  skin_color = 2
  # … the PlayerSettings schema, exactly as in settings.toml
  ```

  The pre-Fox half also lifts the `PreFox` refusal and adds its keys to the spec's types.
- **The compiler's production of the file.** Like the spec's Fox one it is written to the
  output folder with the CPKs a compile publishes, its header naming the CPKs; a compile with
  nothing to put in it (no compiled player, no autopilot export) writes none and moves an
  earlier patch to `aesthetics_patch.toml.bak`, one backup replaced each time, and a run that
  publishes nothing leaves the previous CPK and its patch alone. The compiler's own savefile
  stage is "apply the patch just produced to the local save" (see "Savefile update" in the
  [Team compiler plan](../team_compiler/pipeline.md)) — one path, so a patch applied a week
  later by someone else yields the same bytes the DLC builder's machine wrote.
- **The save editor's apply path.** Applying is the only aesthetics write path the save
  editor leaves open by default (see "Read-only aesthetics" in the
  [Save editor plan](../save_editor.md)); it goes through the editor's undo pipeline and the
  comparator's preview like any other bulk edit.

### Texport (read + write)

What exists of this is in `spec/pes_savefile/operations.md` "Texport (read + write)".

## Player section population (Phase 19, DB generator)

The savefile half of the [DB generator](../db_generator.md), specified here and implemented in
that phase: from PES 19 the game writes a fresh `EDIT00000000` from the database with the player count declared and **no player
records**, so the day-0 save has to be given its placeholder players before anyone (the game
included) can edit them. The scripts do it with a hex editor; here it is one operation:

```rust
/// Writes one base player per (team, slot) into the player section, ids `team*100 + 1..=n`
/// in team order, and sets the count. `overwrite = false` refuses a section that already holds
/// records (the day-0 save of a cup that was hand-populated is not to be flattened by accident).
pub fn populate_players(
    file: &mut EditFile,
    teams: &[TeamId],
    per_team: u8,
    base: &PlayerEntry,
    overwrite: bool,
) -> Result<usize, PopulateError>;
```

Rules: `base` is the version's **base player**, the `PlayerEntry` the scripts' EDIT-layout
templates decode to (`Player_Edit_Base_NN.bin` + `PlayerAppearance_Base_16.bin` assembled as
`player_edit.py` does; committed as a fixture with its provenance), and the operation patches
only `id` and `base_copy_id` (the record's own id copy, `base_copy_id == id` meaning "unset"),
and from Phase 5 the appearance block's player id, set to `id` as real saves hold it (a
stripped save gets its -1 from the Save editor's strip-and-seed, never from here);
`per_team` is capped at the version's roster width; the payload's player section grows to
`teams × per_team` records through `EditFile` — the one place player records are added, since
`to_bytes` otherwise keeps the file's own record count and order; on 15–18 the operation is
allowed but pointless (their fresh saves carry the players) and says so through a finding. The
PES 20 fixture is the day-0 save of a database these scripts generated, so populating a copy
with its player section stripped must reproduce it byte for byte.

---
