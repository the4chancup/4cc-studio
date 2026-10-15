# 4cc Studio — Savefile plan: Operations

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Cross-version player conversion

### What the Rust module is

What exists of this is in `spec/pes_savefile/operations.md` "What the Rust module is".

Not in this module (open): the motion ranges narrow from 20/21 to 19 and earlier (corner kick
1–10 vs 1–6, free kick 1–20 vs 1–16, penalty 1–7 vs 1–4, see the `settings.toml` key table); a
20/21 → 19 conversion copies such a value through uncapped. What the game does with a stored
motion past its range was measured on PES 18, from a patched save: a hunching of stored 3 (the
field is 2 bits; the editor offers 1 to 3, stored 0 to 2) shows "3", a corner kick of stored 7
(3 bits, 1 to 6) shows "6", a goal celebration of 150 shows "127": the list's top, with no
crash on load or in Edit mode. Saving the team with another row of the same screen edited kept
the hunching 3 and the celebration 150 but rewrote the corner kick 7 to 0, the list's first
entry, its own row untouched; PES 16 keeps a stored celebration of 183 the same way. Which
values a save keeps is per field, so a value past the range is not stable. The conversion
therefore caps a motion at the target's top instead of copying it through: the top is what the
game shows for it anyway, and a copied value can turn into the list's first entry on the
player's next in-game save, a change nobody asked for. Each version's tops are measured
(`aesthetics_export/settings_toml.md` "The motion ranges were measured").

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

| Face type | PES 15 | PES 16–18 | PES 19 | PES 20–21 |
|---|---|---|---|---|
| cheek | 14 | 3 | 5 | 3 |
| forehead | 6 | 5 | 6 | 5 |
| facial hair | 29 | 12 | 12 | 19 |
| laughter lines | 6 | 4 | 4 | 4 |
| upper eyelid | 14 | 6 | 6 | 7 |
| lower eyelid | 14 | 2 | 2 | 6 |
| eyebrow | 17 | 5 | 5 | 7 |
| neck line | 5 | 2 | 2 | 3 |
| nose | 11 | 6 | 6 | 7 |
| upper lip | 7 | 3 | 3 | 4 |
| lower lip | 7 | 2 | 2 | 4 |

The PES 15 column joins the editor's range and the census of the game's faces. Neck line 4 and 5
and lips 7 have no editor entry, but the game draws them as distinct models. PES 17 and 18 stay
assumed equal to 16. PES 19's cheek and forehead were measured in its editor (26 cup players on
the maintainer's save use the types past 16's caps); its other nine types stay assumed equal to
16. PES 19 is therefore wider than 20 and 21 on those two, as measured: a 19 to 20 conversion
resets a cheek 4 or 5 like any face type past the target's cap.

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
  below, as the only route for aesthetics, names likewise only from a `write_names` export: per
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
  nothing to put in it (no compiled player) writes none and moves an
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

**The 15-17 team and roster records, measured (Phase 5 converge).** Over every 15-17 texport on
the reference machine (12 PES 16, 34 PES 17; none of PES 15), the 15-17 body lays its records
out as an 18-21 one does, each the save schema's size: on 16, the team record (456 bytes) at
`0x1002C`, then the roster record (164) at `0x101F4`, then the tactics record at `0x10298`; on
17, the team record (480) at `0x10054`, the 88-byte coach block at `0x10234`, the roster record
at `0x1028C`, the tactics record at `0x10330` (the four team-id positions the spec lists are
these records' ids). In all 46 files the team record's, the roster's and the tactics record's
ids agree, the roster holds the 23 player ids and their shirt numbers, and on 16 the player
records follow the roster's order. That order is not always the ids' order: one real PES 16
file (team 804, the fixture `pes16_texport_*`) lists 80401, 80402, 80423, 80404, ..., and the
consecutive-id scan the reader uses today stops after two players and drops the other 21
without an error. PES 15 is taken to lay out as 16 does (the same tactics offset and record
sizes), unverified.

So a 15-17 texport is read and written as an 18-21 one is: `roster_at` is the tactics offset
minus the roster record's size, the team record ends where the roster starts (17: where the
coach block starts, 88 bytes before it); the team, roster and tactics records are read with the
save's codecs (names included), and each filled roster slot's player record is read at that
slot's position (`players_at` + slot × the record stride, as on 18-21), its id checked against
the slot's (`RosterMismatch`, as for 18-21). No real roster has an empty slot before a filled
one (55 texports, 2026-10-15), so slot and packed positions agree on every file measured; a
gapped roster laid out packed would be refused by that id check, never misread. The writer
writes the team and roster records back and lays the players out in the roster's order. All 46
PES 16 and 17 texports read with 23 players, and write back to their own payload byte for byte. We read the records rather than keep the target save's team and roster because
the file carries them, and the consecutive-id rule they replace was ours, never measured: it is
what lost the 21 players. `to_team_toml` on 15-17 then carries the `[team]` section and shirt
numbers as on 18-21.

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
