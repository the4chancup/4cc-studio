# 4cc Studio — Savefile plan: Operations

Part of the [Savefile plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Cross-version player conversion

The converters' `convertPlayerSaveData` becomes a `convert` module: rewrite a
player's appearance data from one version's layout to another's, preserving what
translates and capping what doesn't. Used by the integrated
[model conversion](../model_conversion/README.md) (when compiling an export against a
different PES version's savefile) and by the save editor's cross-version team
import.

What the reference implementations do (19→16 in `convertTeam.py`, 16→21 in
`convertTeam21.py` — for the latter, mind the ⚠ offset warning in the payload
section: only its 240-anchored appearance writes are correct, and the Rust port
re-bases the remaining fields per the schema tables):

- **Copy through**: player name, shirt name, physique bytes, ingame-face block,
  glasses/sleeves/inners/socks/undershorts/shirttail strip settings.
- **Rewrite**: boots/gloves IDs (against the models actually present — defaults
  55/11 hide them, 0 shows defaults), the 4-bit "edited" flags (0x0C with a face
  model, 0x0F without), base-copy ID and taping fields cleared.
- **Cap facial features to the target version's valid ranges** (each version
  raised some maxima; downgrading must clamp): cheek/forehead/facial hair/
  laughter lines/eyelids/eyebrow/neck line/nose/lip types each have per-version
  maxima (e.g. facial hair: 12 in 16, 19 in 21; skin color 7 resets to 1). The
  caps live in a per-version limits table next to the schemas.

Playstyle and skill translation for gameplay data reuses the `menu_lists.cpp`
conversion maps and truncates/zeroes skills that don't exist in the target
version (as 4ccEditor's Texport import already does).

### What the Rust module is

The converters were also the compiler of their day: half of what they write (boots/gloves IDs
against the models present, the edit flags, taping and player gloves cleared) is compile policy,
not layout translation, and in this suite that policy belongs to the Team compiler's
`PlayerSettings`/`ops::fpc` pass that runs *after* conversion. `convert.rs` therefore does the
version-dependent half only: **everything that translates one-to-one is copied through; what does
not translate is capped, dropped or filled from the target, and each such case that depends on
the player's data is reported as a note**, never silently. What depends on the version pair
alone is not a note: a gated field the target version lacks, or the four tail bytes a 16+
template keeps against a PES 15 source, is the same for every player of that pair and is the
caller's to state once (decision entry, 2.17f). The converters' compile-policy rewrites are not in this module (decision
entry, 2.17f).

The operation is a rewrite **into a template**, as in the reference: the target entry is a player
read from a save of the target version, and its bytes stand in for everything the source has no
counterpart for (gated stats the source version lacks, the ingame-face bytes the PES 15 run does
not have). This is also what settles the 46/50-byte run: the appearance block's layout is
identical on every version up to the iris byte and PES 15's is simply four bytes shorter at the
tail (the reference editor's PES 15 and 16 read walks differ only in the last skip), so the
source run is copied over the target run's prefix, `min(len)` bytes; a PES 15 target drops the
source's last four bytes, a 16+ target keeps its own last four. No byte is invented.

```rust
/// `convert.rs`. Rewrites `target`, a player read from a `to`-version save, from `source`,
/// read from a `from`-version save. All-or-nothing: on `Err` the target is unchanged. The
/// target keeps its own `id` (it is the slot being filled).
pub fn convert_player(
    source: &PlayerEntry, from: PesVersion,
    target: &mut PlayerEntry, to: PesVersion,
) -> Result<Vec<ConvertNote>, ConvertError>;

/// What could not be carried one-to-one; the conversion still succeeded.
pub enum ConvertNote {
    /// A face type above the target's cap was reset to 0 (the converters' default; not clamped).
    FaceTypeReset { field: IngameFaceField, value: u8 },
    /// Skin colour 7 (custom skin) reset to 1: `from` or `to` has no custom skin.
    CustomSkinReset,
    /// The playing style has no value in `to`; set to `None` (0).
    PlayingStyleDropped { style: PlayStyle },
    /// A set skill `to` has no bit for was cleared.
    SkillDropped { index: u8 },
    /// The name or shirt name was cut to the target field's length (at a char boundary).
    TextTruncated { text: PlayerText },
    /// The source run is longer than the target's; its last `bytes` bytes were dropped (50 → 46).
    FaceRunTruncated { bytes: usize },
}

pub enum ConvertError {
    /// The source or the target has no ingame-face run (an entry never read from a record).
    NoIngameFaceRun,
    /// The source's stored playing style is not a value of `from`'s list
    /// (`CodecError::UnknownPlayingStyle`).
    Codec(#[from] CodecError),
}
```

Field by field: `name`/`shirt_name` copied, truncated to the target schema's text length with a
note (the reference cuts at 45/15 silently); `basic`, the non-gated `stats`, `positions` (bar
the style), `skills.com_styles`, `motion`, `edit_flags`, the boots/gloves/base-copy IDs, physique
and strip fields copied; a gated `Option` field is carried only where the target has it
(`target.x.is_some()`), otherwise the target's `None` stands; `skills.skills` copied then every
index the target schema has no `Skill(i)` row for cleared (`RecordSchema::has`); the style goes
through `schema::playstyle::{decode, encode}`; the ingame-face run is copied as above, then the
eleven face types are capped against `schema::limits::face_type_cap(to, field)` and the skin rule
applied. Skin 7 is reset only when `fpc::custom_skin_available` is false for `from` or `to`: a
16 → 17 conversion keeps a partial-hide FPC player's custom skin (both converters reset
unconditionally, but each of their directions has a no-custom-skin side; the reference editor's
import resets to 0, the converters to 1, and the converters are the verified path).

```rust
/// `schema/limits.rs`: the per-version caps, next to the schemas. `None` for the fields that
/// are not face types (gloves, skin, iris).
pub fn face_type_cap(version: PesVersion, field: IngameFaceField) -> Option<u8>;
```

| Face type | PES 15–19 | PES 20–21 |
|---|---|---|
| cheek | 3 | 3 |
| forehead | 5 | 5 |
| facial hair | 12 | 19 |
| laughter lines | 4 | 4 |
| upper eyelid | 6 | 7 |
| lower eyelid | 2 | 6 |
| eyebrow | 5 | 7 |
| neck line | 2 | 3 |
| nose | 6 | 7 |
| upper lip | 3 | 4 |
| lower lip | 2 | 4 |

Measured: the PES 16 column (the 19 → 16 converter's caps) and the PES 21 column (the 16 → 21
converter's). Assumed: 15, 17, 18 and 19 equal 16; 20 equals 21. The `settings.toml` key table's
widest range for each face key is derived from this table (the maximum over versions), so the two
cannot drift.

**Playing styles.** The stored value is a per-version index. The canonical `PlayStyle` enum
(`model/playstyle.rs`) has the 22 values of the PES 20/21 list (`None` plus 21 named styles);
`schema/playstyle.rs` holds one list per version group and `decode(version, u8) ->
Result<PlayStyle, CodecError>` / `encode(version, PlayStyle) -> Option<u8>` (`None` when the
version has no such style). The lists, from the reference editor's twelve conversion arrays and
checked against every fixture save (a census of style × registered position: goalkeepers sit at
17/18 on PES 15/16, 16/17 on 17/18, 20/21 on 19–21):

| Index | PES 15/16 | PES 17/18 | PES 19 | PES 20/21 |
|---|---|---|---|---|
| 0–3 | None, Goal Poacher, Dummy Runner, Fox in the Box | same | same | same |
| 4 | Prolific Winger | Prolific Winger | Target Man | Target Man |
| 5 | Classic No. 10 | Classic No. 10 | Creative Playmaker | Creative Playmaker |
| 6 | Hole Player | Hole Player | Prolific Winger | Prolific Winger |
| 7 | Box to Box | Box to Box | Roaming Flank | Roaming Flank |
| 8 | Anchor Man | Anchor Man | Crossing Specialist | Crossing Specialist |
| 9 | The Destroyer | The Destroyer | Classic No. 10 | Classic No. 10 |
| 10 | Extra Frontman | Extra Frontman | Hole Player | Hole Player |
| 11 | Offensive Fullback | Offensive Fullback | Box to Box | Box to Box |
| 12 | Defensive Fullback | Defensive Fullback | The Destroyer | The Destroyer |
| 13 | Target Man | Target Man | Orchestrator | Orchestrator |
| 14 | Creative Playmaker | Creative Playmaker | Anchor Man | Anchor Man |
| 15 | Build Up | Build Up | Build Up | Offensive Fullback |
| 16 | *(unused: no player of either fixture save carries it)* | Offensive Goalkeeper | Offensive Fullback | Fullback Finisher |
| 17 | Offensive Goalkeeper | Defensive Goalkeeper | Fullback Finisher | Defensive Fullback |
| 18 | Defensive Goalkeeper | — | Defensive Fullback | Build Up |
| 19 | — | — | Extra Frontman | Extra Frontman |
| 20 | — | — | Offensive Goalkeeper | Offensive Goalkeeper |
| 21 | — | — | Defensive Goalkeeper | Defensive Goalkeeper |

A stored value outside the list (PES 15/16's 16 included) decodes to
`CodecError::UnknownPlayingStyle { version, value }`, not to `None`: no real save carries one, and a silent 0 is the mistake `Missing` exists to avoid.
The reference editor fills its PES 15/16 combo box with the 17/18 names (mislabelling the two
goalkeeper styles by one); the arrays, not the combo box, are the evidence, and the census confirms
them. The model keeps `PlayerPositions.playing_style: u8` (the stored index): the codec does not
know the version, and only conversion and the interchange formats need the canonical value.

Not in this module (open): the motion ranges narrow from 20/21 to 19 and earlier (corner kick
1–10 vs 1–6, free kick 1–20 vs 1–16, penalty 1–7 vs 1–4, see the `settings.toml` key table); a
20/21 → 19 conversion copies such a value through uncapped. Whether the game clamps, ignores or
misbehaves on an out-of-range motion is unmeasured; measure before capping.

---

## Save-to-save operations

Ported from Midcupping, exposed in the [save editor](../save_editor.md):

- **Aesthetics transplant** (`transplant-aesthetics-*.py`): copy the appearance
  data of selected players from a donor save into a target save of the same
  version. Selection patterns from the scripts, kept as-is: single player ID,
  `target:source` ID pairs, whole team (`team_id` → its 23 player slots), and
  team ranges. In-place or to a new file.
- **Aesthetics fingerprint** (`compare-saves-*.py`): decode a player's aesthetics
  into a comparable struct (boots/gloves/face IDs, taping, glasses, sleeves,
  inners, socks, undershorts, shirttail, winter gloves, skin color) plus a hash
  of the ingame-face block **normalized** by masking the player-gloves and
  skin-color bits (they overlap other fields). This powers the save editor's
  aesthetics diff and gives cheap "did anything visual change" comparisons.

Both are thin operations over the codec — they live here so the save editor and
any batch CLI use identical logic.

---

## Interchange formats

All player/team interchange formats live in this crate, next to the model they
serialize (the save editor is just UI over them):

### Team TOML (new, replaces `.4ccs`/`.4cct`)

A human-readable full-fidelity dump of one team: team entry (names, colors,
manager/stadium IDs, kit slots), tactics (all 3 presets, formations, instructions,
lineup), and every player with **all** known fields — gameplay stats, skills,
positions, motion, edit flags, and the complete aesthetics including the
ingame-face parameters (which the legacy formats omit). Goals:

- Readable and editable in a text editor without opening 4cc Studio.
- The per-player authorable settings use the exact `PlayerSettings` schema from `settings.toml`.
  Full player records additionally retain boots/gloves IDs for save interchange; these fields
  are not part of the aesthetic export's settings schema.
- Version-gated fields serialize as optional keys; importing into an older
  version applies the cross-version conversion (caps + playstyle maps) with
  warnings.
- **Absent = untouched on import**, at every level (key, player, team section) — the same rule
  the aesthetics patch follows. An export writes every field, so a round trip is full-fidelity;
  a file written by hand or by the Team creator carries only what its author decided, and the
  save's values stand in for the rest. A partial file is therefore a patch, not a template with
  holes.
- `shirt_name_from(name, version)` lives here: the shirt name derived from a player name under
  the version's field length and character set. Used by the Team creator's defaults and offered
  by the Save editor as a one-click fill; the lib owns the limits, so no tool carries a copy.

### Aesthetics patch (new)

The file the Team compiler writes on **every** compile and the save editor applies: the resolved
savefile writes for every compiled player, so that the people who build the DLC and the people who
build the official savefile can be different people. The DLC builder compiles the cup's exports
without ever seeing the savefile builder's data (the teams' custom tactics, which they send to the
savefile builder alone), hands over the CPK and the patch, and the savefile builder applies the
patch to the official save. Without the patch, the only way to get compile-time settings into a
save was to compile against that save.

```toml
# aesthetics_patch.toml — written by the Team compiler beside its output CPK
pes_version = 21
studio_version = "1.4.0"
allocation_scheme_version = 1
cpk = "4cc_69.cpk"
compiled = 2026-09-12T14:03:00Z

[[teams]]
id = 701
name = "/a/"                 # for the reader; the id is what is applied

[teams.players.03]           # slot 03 → player 70103
name = "Snuffy"              # resolved: `name = true` became the string
boots_id = 28043             # compiler-owned fields, as assigned
gloves_id = 0
edit_flags = 12

[teams.players.03.appearance]
skin_color = 2
# … the PlayerSettings schema, exactly as in settings.toml
```

Rules:

- **Per player, the file carries the `PlayerSettings` schema plus the compiler-owned fields**
  (`boots_id`, `gloves_id`, `edit_flags`) — the same union Team TOML's player aesthetics section
  holds; nothing is invented for the patch. Only fields the compile resolved to a write appear;
  **absent = untouched** at every level (field, player, team), which is what lets a midcup patch
  covering three teams apply to a save cleanly after a full-cup one.
- **Everything is resolved.** `name = true` is the derived string; `fpc.on`/`fpc.off` are the
  preset's concrete field values; the IDs are the assigned ones, already filtered by the compiler's
  "only for content that was actually packed" rule. The patch answers "what will the save contain",
  never "what did the export say" — the export is not needed to apply it.
- **Applying is `pes_savefile`'s operation** (`apply_patch`): for each player, apply the
  `PlayerSettings` and set the compiler-owned fields, through the same code the compiler's own
  savefile stage uses — because that stage *is* "apply the patch just produced to the local save"
  (see "Savefile update" in the [Team compiler plan](../team_compiler/README.md)). One path, so a patch applied
  a week later by someone else yields the same bytes the DLC builder's machine wrote.
- `pes_version` must equal the save's; there is no cross-version application — the compile targeted
  a version and its IDs and presets are that version's. `allocation_scheme_version` must equal the
  suite's; a patch from an older scheme is refused with the pointer to the from-scratch remake rule.
  A team ID absent from the save skips that team with an error and applies the rest; referees are
  team 999 like anywhere else.
- Applying is the only aesthetics write path the save editor leaves open by default (see "Read-only
  aesthetics" in the [Save editor plan](../save_editor.md)); it goes through the editor's undo pipeline
  and the comparator's preview like any other bulk edit.

### Legacy 4ccEditor formats (read-only)

- **`.4ccs` squad files** (version tag "21a"): binary `player_export` array +
  shirt numbers + optional trailing tactics block (the same version-neutral
  layout as `.4cct` below, written when the save's version has tactics support —
  16–21 — and gated by an import-time checkbox). Readable and importable; never
  written.
- **`.4cct` "nightly" tactics files** (version tag "001"): `"001"` + 2-char PES
  version + 8-char team ID + a version-neutral tactics block (405 bytes: per
  preset, the 3×3 formations, 7 style bytes, 2+2 advanced instructions each as
  canonical-enum byte + player byte, 5 sliders + fluid; then starting XI,
  21 bench, 6 set-piece takers, 3 join-attack, 4 auto flags — **no captain**;
  instructions stored in the canonical 17-based enum and re-encoded per target
  version on load). Readable and importable; never written. Import targets PES
  16–18 only (4ccEditor's own gate — later saves go through Texport instead),
  and the file header records the exporting PES version, which the import must
  match exactly.

Both parse into the same model structs; the write path is Team TOML only.

### Texport (read + write)

Texport files are **PES's own in-game team export/import format** — a different
internal layout from the savefile (tactics record first, then player entries;
4ccEditor's `handle_texport` + `fill_player_entryXX_texport`/
`fill_team_tacticsXX_texport` are the reference). The crypto splits by era:
15–17 `.ted` files use the same container crypto as that version's savefile
(`decryptFile15` / `decryptWithKeyOld` + master keys), while 18–21 use a
self-contained XOR scheme instead — 0x30-byte header, a 0x20-byte key at 0x30,
payload from 0x50 XOR'd against the key starting at index 0x12/0x13/0x14/0x15
(18/19/20/21) and wrapping at 0x20 (decrypted sizes 0x1FC0/0x25B0/0x39E4; the
PES 20 key index is 4ccEditor's untested guess). The tactics record is the save
record layout verbatim, including its 4-byte team ID; player entries carry the
player's own record plus appearance data inline (17's reader consumes no
appearance — whether absent or skipped is unverified). Per-version record
offsets:

| Version | Tactics | Players |
|---------|---------|---------|
| 15 | 0x10298 | 0x51065C |
| 16 | 0x10298 | 0x510660 |
| 17 | 0x10330 | 0x510840 |
| 18 | 0x32C | 0x834 |
| 19 | 0x33C | 0x844 |
| 20/21 | 0x410 | 0x918 |

Import never trusts the file's player IDs: players map onto the target team's
roster by slot order (`team_id*100 + 1 + slot`), and a 15/16 import zeroes the
stats that don't exist yet (play_skill 28–41, tight_pos/aggression/physical =
77) — the same defaults cross-version conversion applies.
Because the game itself consumes these, full compatibility matters in both
directions: read (4ccEditor already does) **and write** (new — 4ccEditor is
import-only), so the editor can produce a file PES can import. Write support
needs the full texport layout, not just the fields 4ccEditor reads; transcribe it
per version and verify by importing the output into the game. If full-layout
transcription proves impractical for some version, the fallback is
template-patching: load a real texport of that version and rewrite the fields we
model.

---
