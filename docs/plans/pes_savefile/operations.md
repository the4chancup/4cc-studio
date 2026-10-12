# 4cc Studio — Savefile plan: Operations

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Cross-version player conversion

### What the Rust module is

What exists of this is in `spec/pes_savefile/operations.md` "What the Rust module is".

Not in this module (open): the motion ranges narrow from 20/21 to 19 and earlier (corner kick
1–10 vs 1–6, free kick 1–20 vs 1–16, penalty 1–7 vs 1–4, see the `settings.toml` key table); a
20/21 → 19 conversion copies such a value through uncapped. Whether the game clamps, ignores or
misbehaves on an out-of-range motion is unmeasured; measure before capping.

## Interchange formats

### Team TOML (new, replaces `.4ccs`/`.4cct`)

What exists of this is in `spec/pes_savefile/operations.md` "Team TOML (new, replaces `.4ccs`/`.4cct`)".

An import never sets a player's appearance id:
a stripped save stays stripped (see "Stripped save" in the [Save editor plan](../save_editor.md)).

These keys extend the spec's `team.toml` block:

```toml
[players.01]
height = 180                    # cm, the record's own field
weight = 75                     # kg
stripped = true                 # Fox: written only for a player whose appearance id is -1 ("Stripped save"); informational, import ignores it

[players.01.appearance]
# (.unknown's face: the ingame-face run's undecoded bits; absent = untouched on import, as every key)
```

These fields extend the spec's `PlayerSection`:

```rust
    pub height: Option<u8>, pub weight: Option<u8>,
    pub stripped: Option<bool>,  // written for a player at -1; informational, ignored on import
```

### Aesthetics patch (new)

The file the Team compiler writes to its output folder with its CPKs and the save editor applies: the resolved
savefile writes for the compiled players, so that the people who build the DLC and the people who
build the official savefile can be different people. The DLC builder compiles the cup's exports
without ever seeing the savefile builder's data (the teams' custom tactics, which they send to the
savefile builder alone), hands over the CPK and the patch, and the savefile builder applies the
patch to the official save.

**What it holds depends on the engine.** On **Fox** (PES 18–21) a player's appearance, boots and
gloves travel in the CPK's database tables, not in the savefile (see "Player settings in exports"
in the [Aesthetics export plan](../aesthetics_export/settings_toml.md)), so the patch holds what
the tables cannot carry: every compiled player's **motions** (`[appearance.motion]`, player-record
fields outside the appearance block a table row holds) and the **names and shirt names** of an
**autopilot** export's players, one with the `autopilot` marker at its root (its tactical export
carries placeholder names). A patch's `players.NN` is player id `team id × 100 + NN`, the
export's savefile identity (`spec/aesthetics_export/player_folders.md` "Savefile identity"), whatever roster slot he
holds, so the patch lands on the player whose models the CPK carries. It is applied whole, with
no team picker: a managed team's export has
no marker, so its `name` and `shirt_name` are not used and its players keep the names their
manager set. Like the pre-Fox one it is written to the output folder with the CPKs a compile
publishes, its header naming the CPK; a compile with nothing to put in it (no compiled player,
no autopilot export) writes none and moves an earlier patch to
`aesthetics_patch.toml.bak`, one backup replaced each time, and a run that publishes nothing
leaves the previous CPK and its patch alone. On **pre-Fox** (PES 15–17) it holds everything
below, as the only route for aesthetics, names likewise only from an autopilot export.

```toml
# aesthetics_patch.toml, written by the Team compiler to its output folder
pes_version = 21
studio_version = "1.4.0"
allocation_scheme_version = 1
cpks = ["4cc_69.cpk"]          # the CPKs the run published; none for a sideload tree
compiled = 2026-09-12T14:03:00Z

[[teams]]
id = 702
name = "/a/"                 # for the reader; the id is what is applied

[teams.players.03]           # slot 03 → player 70203
name = "Snuffy"              # resolved: `name = true` became the string
shirt_name = "SNUFFY"
# pre-Fox only from here on, except [teams.players.03.appearance.motion], which a Fox patch holds
boots_id = 143               # resolved: the assigned custom ID, the authored stock ID or the marker's default
gloves_id = 0
edit_flags = 12

[teams.players.03.appearance]
skin_color = 2
# … the PlayerSettings schema, exactly as in settings.toml, unknown bits included
```

Rules:

- **Per player, a pre-Fox file carries the `PlayerSettings` schema plus `name`, `shirt_name` and
  the compiler-owned `edit_flags`**, with `boots_id`/`gloves_id` over their full stored range: the
  resolved ID is the one the order in the [Aesthetics export plan](../aesthetics_export/settings_toml.md)'s
  "Player settings in exports" reaches (a committed custom output's assigned ID, else the authored
  stock ID 0 to 100, else the FPC marker's default), and a category whose requested output failed
  carries no key, so the save keeps its current ID — the same union Team TOML's player aesthetics
  section holds; nothing is invented for the patch. Every appearance field is written, with
  `settings.toml`'s defaults for keys its file lacks, so a player's appearance is his export's
  whatever the save held. A Fox file carries every compiled player's motion keys, with the
  defaults for keys his file lacks, and an autopilot export's `name` and `shirt_name`. On both
  engines `name` and `shirt_name` come only from an autopilot export. Absent players and
  teams are untouched, which is what lets a midcup patch covering three teams apply to a save
  cleanly after a full-cup one; an absent `name` or `shirt_name` is not written.
- **A Fox patch keeps a stripped player stripped.** Applying it changes the motion and name
  fields alone; the appearance block, and its player id at -1, are left as they are, so the
  player keeps taking his appearance from the database tables (see "Stripped save" in the
  [Save editor plan](../save_editor.md)).
- **Everything is resolved.** `name = true` is the derived string; `fpc_on`/`fpc_off` are the
  preset's concrete field values; the IDs are the resolved ones, already filtered by the compiler's
  "only for content that was actually packed" rule. The patch answers "what will the save contain",
  never "what did the export say" — the export is not needed to apply it.
- **Applying is `pes_savefile`'s operation** (`apply_patch`): for each player, apply what the
  file holds (`PlayerSettings`, IDs and edit flags on pre-Fox; motions and names on Fox), through the same
  code the compiler's own
  savefile stage uses — because that stage *is* "apply the patch just produced to the local save"
  (see "Savefile update" in the [Team compiler plan](../team_compiler/pipeline.md)). One path, so a patch applied
  a week later by someone else yields the same bytes the DLC builder's machine wrote.
- `pes_version` must equal the save's; there is no cross-version application — the compile targeted
  a version and its IDs and presets are that version's. `allocation_scheme_version` must equal the
  suite's; a patch from an older scheme is refused with the pointer to the from-scratch remake rule
  (pre-Fox's consequence; the check is the same on both engines, a Fox patch carrying no IDs).
  A team or player ID absent from the save is skipped with an error and the rest applied; referees are
  team 999 like anywhere else.
- Applying is the only aesthetics write path the save editor leaves open by default (see "Read-only
  aesthetics" in the [Save editor plan](../save_editor.md)); it goes through the editor's undo pipeline
  and the comparator's preview like any other bulk edit.
- **A patch is refused whole or applied whole, absences aside.** Every check that can fail
  (the two versions, a pre-Fox `pes_version` until the pre-Fox half is built, a name longer
  than the version's field, a motion value outside its key's range, a `dribbling` for a version
  without it) runs before the first write, and a failure leaves the save untouched: the patch
  is the compiler's output, so a bad value means a broken patch, and a half-applied one would
  leave a save matching no compile. Only a team or player absent from the save is skipped per
  item, as above.

The Fox half's types (`interchange/aesthetics_patch.rs`; the pre-Fox half adds its keys when it
is built):

```rust
/// The suite's boots and gloves ID allocation scheme (`team_compiler/pipeline.md`); a patch
/// from another scheme is refused.
pub const ALLOCATION_SCHEME_VERSION: u32 = 1;

/// `aesthetics_patch.toml`: the compiler's resolved savefile writes for its compiled players.
pub struct AestheticsPatch {
    pub pes_version: PesVersion,
    pub studio_version: String,
    pub allocation_scheme_version: u32,
    /// The CPKs the run published; empty for a sideload tree.
    pub cpks: Vec<String>,
    pub compiled: toml_edit::Datetime,
    pub teams: Vec<PatchTeam>,
}

pub struct PatchTeam {
    pub id: u32,
    /// For the reader; the id is what is applied.
    pub name: String,
    /// By `NN` (`players.NN`, player id `id × 100 + NN`).
    pub players: BTreeMap<u8, PatchPlayer>,
}

pub struct PatchPlayer {
    /// An autopilot export's only; `None` = not written.
    pub name: Option<String>,
    pub shirt_name: Option<String>,
    /// `[appearance.motion]`, as in `settings.toml`; `None` keys are not written.
    pub motion: MotionSettings,
}

impl AestheticsPatch {
    pub fn parse(text: &str) -> Result<Self, PatchError>;
    /// `Err` for a value `parse` would refuse back (a hand-built one).
    pub fn to_toml(&self) -> Result<String, PatchError>;
    /// Every refusal before any write (above); then each team and player present is written.
    pub fn apply(&self, save: &mut EditFile) -> Result<PatchSummary, PatchError>;
}

pub struct PatchSummary {
    pub applied_teams: Vec<u32>,
    pub skipped: Vec<PatchSkip>,
}

pub enum PatchSkip {
    TeamNotInSave { team: u32 },
    PlayerNotInSave { player: u32 },
}

/// The refusals, each naming what failed; a key-level one (unknown, wrong type, out of
/// range, a name too long) in `settings.toml`'s vocabulary, at the key's dotted path.
pub enum PatchError {
    Toml(String),
    MissingKey { key: String },
    PreFox { version: PesVersion },
    VersionMismatch { patch: PesVersion, save: PesVersion },
    SchemeMismatch { patch: u32, suite: u32 },
    Settings(SettingsError),
}
```

### Texport (read + write)

What exists of this is in `spec/pes_savefile/operations.md` "Texport (read + write)".

Every player record's appearance id holds the player's own id in each export the game wrote
(the 18-21 fixtures), so the writer writes it so, whatever the save's entry holds: a stripped
player's -1 is the save's state, not the export's.

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
