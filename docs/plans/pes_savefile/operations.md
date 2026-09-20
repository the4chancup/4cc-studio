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

Ported from Midcupping's scripts and the reference editor's comparator, exposed in the
[save editor](../save_editor.md):

- **Aesthetics transplant** (`transplant-aesthetics-*.py`): copy the appearance
  data of selected players from a donor save into a target save of the same
  version. Selection patterns from the scripts, kept as-is: single player ID,
  `target:source` ID pairs, whole team (`team_id` → its 23 player slots), and
  team ranges. In-place or to a new file.
- **Aesthetics fingerprint** (`compare-saves-*.py`): a hash of the ingame-face block
  **normalized** by masking the player-gloves and skin-color bits (they overlap fields the
  script lists separately): the first four bytes of SHA-256, printed as eight hex characters
  as the script does. It is how the comparator says "the face changed" without decoding every
  facial parameter, and how a fingerprint printed by the script can be checked against ours.
- **Comparator** (the reference editor's `comparator.cpp` plus `compare-saves-*.py`): every
  stored field of two same-version players, old → new, each row tagged gameplay or aesthetics.

All three are thin operations over the model: they live here so the save editor and any batch
CLI use identical logic. Per-entry functions do the work; the save-level entry points resolve
ids and check the version pair, and take `EditFile`s (the decoded save; `ops/` still never
touches bytes).

### What the reference scripts do

The transplant script rewrites one byte slice of the target's appearance block from the donor's:
bytes 4 to 68 of the 72-byte block (`output[4:68] = playerFace[4:68]`; on PES 15 the block is 68
bytes and the slice is everything but the id; on 17+ the block sits inside the player record and
the same slice applies). That slice is the four appearance edit flags (face, hair, physique,
strip), boots/gloves IDs, the base-copy id, the fourteen physique measures, the strip settings
and the first 46 bytes of the ingame-face run. It does *not* copy the id, and on 16/17 it leaves
the run's last four bytes (a PES 15 slice never adjusted). Measured: those four bytes are zero on
every one of the 24 656 players of the 16/17/18/19/21 fixtures, so copying the whole block is
byte-identical to the script on real saves and one rule instead of two (decision entry, 2.17g).
The donor's base-copy id is copied as it stands (the donor's own id when it has its own face):
that is the script's behavior and the game's meaning of a face transplant.

The compare script decodes, per player, boots/gloves/face IDs (face = base-copy id, or 0 when it
equals the player's own id), the two wrist tapings, spectacles style, sleeves, inners, socks,
undershorts, shirttail, ankle taping, player gloves and colour, skin, and the normalized run's
hash; the reference editor's comparator walks each team's roster slots and lists names, id, age,
height, weight, every ability, style, registered position, stronger foot, form, injury, weak
foot, playable positions, COM styles and skills. Neither compares motion, edit flags, nationality
or the physique measures; ours compares every stored field, so the two references are each a
subset of the rows.

```rust
/// `ops/transplant.rs`. The per-entry operation: `target` takes `donor`'s appearance block
/// (`appearance`, ingame-face run included, and the face/hair/physique/strip edit flags);
/// everything else, the id included, is the target's own.
pub fn transplant_player(target: &mut PlayerEntry, donor: &PlayerEntry);

/// The save-level operation over `(target_id, donor_id)` pairs; all-or-nothing: the version
/// pair and every id are checked before the first write.
pub fn transplant(
    target: &mut EditFile, donor: &EditFile, pairs: &[(u32, u32)],
) -> Result<(), TransplantError>;

pub enum TransplantError {
    VersionMismatch { target: PesVersion, donor: PesVersion },
    /// A pair names a player the target save lacks.
    TargetMissing(u32),
    /// A pair names a player the donor save lacks.
    DonorMissing(u32),
}

/// One selection argument in the scripts' four forms, expanded to `(target, donor)` pairs:
/// `70103` (one player, onto itself), `70103:70205` (target:donor), `701` (a team: its
/// slots 1–23, `team * 100 + 1..=23`, onto themselves), `701-720` (a team range, likewise).
/// A number below 1000 is a team, as in the scripts.
pub fn parse_selection(arg: &str) -> Result<Vec<(u32, u32)>, SelectionError>;

pub enum SelectionError {
    /// Not a number, or a pair/range with more than one separator.
    Malformed(String),
    /// A range whose ends are not teams (≥ 1000) or run backwards.
    BadRange(String),
}
```

```rust
/// `ops/fingerprint.rs`. The first four bytes of SHA-256 over `IngameFace::normalized()`;
/// `Display` is the eight lower-case hex characters the reference prints.
pub struct FaceHash(pub [u8; 4]);
pub fn face_hash(face: &IngameFace) -> FaceHash;
```

```rust
/// `ops/compare.rs`. Every difference between two players of the same version: the schema's
/// stored fields (the player record's, plus the 15/16 appearance record's) through the model,
/// the two texts, the known ingame-face bits, and the run's undecoded bits as one row.
/// `Err` only when a gated field the schema stores is `None` on either side (an entry not
/// read from a save of this version).
pub fn compare_players(
    a: &PlayerEntry, b: &PlayerEntry, schema: &VersionSchema,
) -> Result<Vec<PlayerDiff>, CodecError>;

pub enum PlayerDiff {
    /// A bit-run field of the record.
    Field { field: PlayerField, old: u32, new: u32 },
    /// The name or shirt name.
    Text { text: PlayerText, old: String, new: String },
    /// A known bit run of the ingame-face block.
    Face { field: IngameFaceField, old: u8, new: u8 },
    /// Bits of the run no `IngameFaceField` names differ; the hashes are the reference's
    /// fingerprints of the two runs.
    FaceRun { old: FaceHash, new: FaceHash },
}

impl PlayerDiff {
    /// Aesthetics is what `transplant_player` moves: the appearance-block fields (the four
    /// appearance edit flags, boots/gloves/base-copy IDs, physique, strip), every `Face` and
    /// `FaceRun` row. Everything else (motion and the other edit flags included) is gameplay.
    pub fn scope(&self) -> DiffScope;
}
pub enum DiffScope { Gameplay, Aesthetics }

/// Two saves, players paired by id (the compare script's pairing; the reference editor pairs
/// by roster slot, which reports a reshuffle as edits).
pub fn compare(a: &EditFile, b: &EditFile) -> Result<SaveDiff, CompareError>;

pub struct SaveDiff {
    /// Players in both saves with at least one difference, in `a`'s record order.
    pub players: Vec<(u32, Vec<PlayerDiff>)>,
    pub only_in_a: Vec<u32>,
    pub only_in_b: Vec<u32>,
}
pub enum CompareError {
    VersionMismatch { a: PesVersion, b: PesVersion },
    Codec(#[from] CodecError),
}
```

A face-type change is reported once, as its `Face` row: `FaceRun` fires only when bits outside
every known field differ, so the row means "something in the face we cannot name changed". The
hash it carries still masks only gloves and skin (`normalized()`, unchanged), so it equals the
script's printed fingerprint. Team and tactics comparison is the save editor's later concern
(`save_editor.md` "Comparator") and not part of this module.

Parity: the transplant is checked against the script's slice rule on every player of every
fixture (target record after `transplant_player` + `write_player` equals the record with bytes
4..68 replaced from the donor record, the id kept); the fingerprint and each Midcupping field
against the script's bit reads, transcribed as literal offsets in the golden test, over every
fixture player; the comparator against itself (a player compared with its own copy has no rows,
a transplant leaves no aesthetics row and the gameplay rows unchanged).

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

**The file.** Every table is addressable by name so that "absent = untouched" holds at every
level (a `[[presets]]` array could not leave preset 2 alone while setting preset 1). Players are
keyed by **roster slot**, never by id: slot `NN` of the file is roster slot `NN` of the team it
is applied to (the aesthetics patch and Texport use the same rule), so a team's file imports
onto any team, and a file never carries a player id. Gameplay values are the **stored** numbers
(the record's, ranges from the bit widths: this is a dump, and the manager-facing reinterpreted
numbers belong to `settings.toml`); what the model already names canonically is written as a
label: positions, playable ratings, the playing style, skills, COM styles, the advanced
instructions, the style switches, the stronger foot/hand. The `[players.NN.appearance]` table is
`settings.toml`'s `[appearance]` table verbatim (same keys, same kinds, same ranges, parsed and
emitted by `settings_toml`'s own parser and emitter), so a player's aesthetics can be pasted
between the two files. One difference of policy, not of keys: Team TOML reads and writes the
*stored* value of every field (real saves hold face types and celebrations past the editor's
ranges, and a dump must carry them), where `settings.toml` keeps the editor ranges (decision
entry, 2.17h-3).

```toml
# team.toml — one team of one save. Absent = untouched on import, at every level.
pes_version = 19                # the save this was written from; import into another version converts

[team]
id = 841                        # informational: the caller chooses the target team
name = "/98hu/"                 # colour codes as in the save
short_name = "I1"
manager_id = 1000               # PES 19+
stadium_id = 12                 # PES 19+
color_1 = [31, 58, 15]          # PES 17+; stored channels, 0-63 each
color_2 = [63, 63, 63]
kit_slots = [{ number = 0, binding = 0 }, ...]   # PES 17 only; ten entries, values as stored

[team.edit_flags]
name = true
short_name = false              # PES 15 only
stadium = false                 # PES 20/21 only
strip = false                   # PES 17 only

[tactics]
starting_eleven = [0, 10, 3, 1, 5, 4, 9, 2, 6, 7, 8]  # roster slots, 0-based as stored
bench_order = [11, 12, ...]     # 21 roster slots
players_to_join_attack = [255, 255, 255]             # roster slots; 255 = none

[tactics.set_pieces]            # roster slots; 255 = none
free_kick_long = 3
free_kick_short = 3
free_kick_second = 4
corner_left = 5
corner_right = 5
penalty = 9
captain = 0

[tactics.auto]
substitution = 0                # PES 16+; the stored setting byte
offside_trap = false            # PES 16+
preset_change = false           # PES 16+
attack_defence_levels = false   # PES 17+

[tactics.preset_1]              # preset_1 .. preset_3
[tactics.preset_1.style]
attacking_style = "possession"  # "counter_attack" | "possession"
attacking_zone = "wide"         # "centre" | "wide"
buildup = "short_pass"          # "long_pass" | "short_pass"
positioning = "flexible"        # "maintain" | "flexible"
defensive_style = "frontline_pressure"   # "frontline_pressure" | "all_out_defence"
containment_area = "middle"     # "middle" | "wide"
pressure = "aggressive"         # "aggressive" | "conservative"
fluid = false                   # PES 16+
[tactics.preset_1.sliders]
support_range = 4               # 1-10
defensive_line = 3              # 1-10
compactness = 7                 # 1-10
numbers_in_attack = 3           # 1-3
numbers_in_defence = 3          # 1-3
[tactics.preset_1.formation_1]  # formation_1 kick-off, formation_2 in possession, formation_3 out of possession
players = [{ position = "GK", x = 52, y = 2 }, ...]  # eleven; position "GK" .. "CF"; x, y as stored
[tactics.preset_1.instructions] # PES 17+; canonical names, re-encoded per version on import
attack = [{ instruction = "hug_the_touchline", player = 3 }, { instruction = "off", player = 0 }]
defence = [{ instruction = "off", player = 0 }, { instruction = "off", player = 0 }]

[players.01]                    # roster slot 01 .. 40 (32 on PES 15-18)
name = "YOUR SKILL: A FAILURE"
shirt_name = "SCORE"
number = 1                      # shirt number, the roster's
nationality = 231
age = 25
boots_id = 3601
gloves_id = 0
base_copy_id = 84101            # equal to the player's own id in the file = unset; becomes the target's own id
ingame_face = "0102...ff"       # the appearance block's undecoded run, hex, 50 bytes (46 on PES 15); applied before [players.NN.appearance]

[players.01.stats]              # stored values; the comments give the game's editor range, the parser accepts the field's stored range
attacking_prowess = 77
ball_control = 77
dribbling = 77
tight_possession = 77           # PES 20+
low_pass = 77
lofted_pass = 77
finishing = 77
heading = 77
place_kicking = 77
swerve = 77
defensive_prowess = 77
ball_winning = 77
aggression = 77                 # PES 20+
kicking_power = 77
speed = 77
explosive_power = 77
body_control = 77
physical_contact = 77           # PES 17+
jump = 77
stamina = 77
goalkeeping = 40
catching = 40
clearing = 40                   # PES 16+
reflexes = 40                   # PES 16+
coverage = 40                   # PES 16+
weak_foot_usage = 1             # 0-3 stored
weak_foot_accuracy = 1          # 0-3 stored
form = 3                        # 0-7 stored
injury_resistance = 1           # 0-2 stored
star = 2                        # PES 19+, 0-7 stored
playing_attitude = 0            # PES 20+, stored

[players.01.positions]
registered = "CF"               # "GK" "CB" "LB" "RB" "DMF" "CMF" "LMF" "RMF" "AMF" "LWF" "RWF" "SS" "CF"
playable = { CF = "A", SS = "B", GK = "none", ... }   # thirteen keys; "none" | "C" | "B" | "A"
playing_style = "goal_poacher"  # canonical `PlayStyle` name in snake_case; "none" for none
stronger_foot = "right"         # "right" | "left"
stronger_hand = "right"         # PES 20+

[players.01.skills]
skills = ["scissors_feint", "heading"]      # the set skills, canonical names (table below)
com_styles = ["trickster"]                  # "trickster" "mazing_run" "speeding_bullet" "incisive_run" "long_ball_expert" "early_cross" "long_ranger"

[players.01.edit_flags]         # the game's own flags, all bool
player = true
basic_settings = true
registered_position = false
playable_positions = false
abilities = true
skills = false
playing_style = false
com_styles = false
motion = false
base_copy = false
face = true
hair = true
physique = false
strip = false

[players.01.appearance]         # settings.toml's [appearance] table, verbatim
skin_color = 1
# ... [players.01.appearance.physique] / .strip / .motion / .face, as in settings.toml
```

Skill names, in the model's slot order 0-40: `scissors_feint flip_flap marseille_turn sombrero
cut_behind_and_turn scotch_move heading long_range_drive knuckle_shot acrobatic_finishing
heel_trick first_time_shot one_touch_pass weighted_pass pinpoint_crossing outside_curler rabona
low_lofted_pass low_punt_trajectory long_throw gk_long_throw malicia man_marking track_back
acrobatic_clear captaincy super_sub fighting_spirit double_touch crossover_turn step_on_skill
chip_shot dipping_shots rising_shots no_look_pass gk_high_punt penalty_specialist
gk_penalty_specialist interception long_range_shooting through_passing`. Advanced instruction
names, in the canonical 17-based order 0-16: `off hug_the_touchline false_no_9 false_full_backs
attacking_full_backs wing_rotation tiki_taka centering_targets swarm_the_box deep_defensive_line
gegenpress tight_marking counter_target defensive false_winger wing_back anchoring`; the
per-version encodings are `schema/instruction.rs` (below). The reference editor's table ends at
`wing_back`; the PES 20 and 21 fixture saves store a value `0x10` in seven instruction slots (the
instruction PES 2020 added; the name is the game's list's, unverified against these saves'
edit screen), canonical 0x10 here and absent before PES 20.

```rust
/// `model/instruction.rs`. The canonical advanced instruction, in the reference editor's
/// 17-based order: 0x00-0x0C are PES 17's own values, 0x0D-0x0F the three PES 18 added,
/// 0x10 the one PES 20 added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction { Off, HugTheTouchline, FalseNo9, FalseFullBacks, AttackingFullBacks,
    WingRotation, TikiTaka, CenteringTargets, SwarmTheBox, DeepDefensiveLine, Gegenpress,
    TightMarking, CounterTarget, Defensive, FalseWinger, WingBack, Anchoring }

/// `schema/instruction.rs`. PES 17 stores the canonical value; PES 18-21 store 0x01-0x07 as is,
/// Defensive/FalseWinger at 0x08/0x09, SwarmTheBox..CounterTarget at 0x0A-0x0E, WingBack at
/// 0x0F; PES 20/21 add Anchoring at 0x10. PES 15/16 have no instructions (`None` for every
/// value). A stored value outside the version's list is
/// `CodecError::UnknownInstruction { version, value }`.
pub fn decode(version: PesVersion, stored: u8) -> Result<Option<Instruction>, CodecError>;
/// `None` when the version has no such instruction (a PES 17 target for the three PES 18 ones,
/// a 19-or-earlier target for Anchoring).
pub fn encode(version: PesVersion, instruction: Instruction) -> Option<u8>;
```

```rust
/// `interchange/team_toml/`. The document: every leaf `Option`, `None` = absent.
pub struct TeamToml {
    pub pes_version: Option<PesVersion>,
    pub team: TeamSection,          // the [team] keys and [team.edit_flags]
    pub tactics: TacticsSection,    // [tactics], .set_pieces, .auto, preset_1..3 with their sub-tables
    pub players: BTreeMap<u8, PlayerSection>,   // roster slot (1-based, as in the key) -> the player's tables
}
/// The sections mirror the tables, field names = key names, every leaf `Option`; what the TOML
/// writes as a label is the model's value here (positions `u8`, switches `bool`, `PlayStyle`,
/// `Instruction`), the label tables being the parser's and emitter's alone.
pub struct TeamSection { pub id: Option<u32>, pub name: Option<String>, pub short_name: Option<String>,
    pub manager_id: Option<u32>, pub stadium_id: Option<u16>, pub color_1: Option<[u8; 3]>,
    pub color_2: Option<[u8; 3]>, pub kit_slots: Option<[KitSlot; 10]>, pub edit_flags: TeamEditFlagsSection }
pub struct TeamEditFlagsSection { pub name: Option<bool>, pub short_name: Option<bool>, pub stadium: Option<bool>, pub strip: Option<bool> }
pub struct TacticsSection { pub starting_eleven: Option<[u8; 11]>, pub bench_order: Option<[u8; 21]>,
    pub players_to_join_attack: Option<[u8; 3]>, pub set_pieces: SetPiecesSection, pub auto: AutoSection,
    pub presets: [PresetSection; 3] }
pub struct SetPiecesSection { /* the seven takers, `Option<u8>` each, names as in `SetPieceTakers` */ }
pub struct AutoSection { pub substitution: Option<u8>, pub offside_trap: Option<bool>, pub preset_change: Option<bool>, pub attack_defence_levels: Option<bool> }
pub struct PresetSection { pub style: StyleSection, pub sliders: SlidersSection,
    pub formations: [Option<[FormationSlot; 11]>; 3], pub instructions: Option<InstructionsSection> }
pub struct StyleSection { /* the seven switches + `fluid`, `Option<bool>` each, names as in `TacticStyle` */ }
pub struct SlidersSection { /* the five sliders, `Option<u8>` each, names as in `TacticSliders` */ }
pub struct InstructionsSection { pub attack: [InstructionEntry; 2], pub defence: [InstructionEntry; 2] }
pub struct InstructionEntry { pub instruction: Instruction, pub player: u8 }
/// One `[players.NN]`.
pub struct PlayerSection { pub name: Option<String>, pub shirt_name: Option<String>, pub number: Option<u16>,
    pub nationality: Option<u16>, pub age: Option<u8>, pub boots_id: Option<u32>, pub gloves_id: Option<u32>,
    pub base_copy_id: Option<u32>, pub ingame_face: Option<Vec<u8>>, pub stats: StatsSection,
    pub positions: PositionsSection, pub skills: SkillsSection, pub edit_flags: EditFlagsSection,
    pub appearance: AppearanceSettings }
pub struct StatsSection { /* one `Option<u8>` per `[players.NN.stats]` key, names as in the block */ }
pub struct PositionsSection { pub registered: Option<u8>, pub playable: Option<[u8; 13]>,
    pub playing_style: Option<PlayStyle>, pub stronger_foot: Option<u8>, pub stronger_hand: Option<u8> }
pub struct SkillsSection { pub skills: Option<[bool; 41]>, pub com_styles: Option<[bool; 7]> }
pub struct EditFlagsSection { /* the fourteen flags, `Option<bool>` each, names as in `PlayerEditFlags` */ }

/// `interchange/legacy.rs`: the read-only formats, each into a `TeamToml` (see below).
pub fn read_squad(bytes: &[u8]) -> Result<TeamToml, LegacyError>;     // .4ccs
pub fn read_tactics(bytes: &[u8]) -> Result<TeamToml, LegacyError>;   // .4cct
pub enum LegacyError { BadTag { expected: &'static str, found: String }, BadVersion(String), Truncated { needed: usize, len: usize },
    UnknownPlayingStyle { version: PesVersion, value: u8 }, Text(String) }

/// What an import did not apply as written. Version-pair-wide facts are not notes (the 2.17f rule).
pub enum ImportNote {
    /// A key the target version has no field for (`tight_possession` into PES 19); dropped.
    NotInThisVersion { path: String },
    /// A label the target version cannot encode (a PES 18 instruction into 17, a style the
    /// target's list lacks); written as `off`/the list's rule.
    NotEncodable { path: String, label: String },
    /// A face type above the target's cap (reset to 0, the 2.17f rule) or skin 7 into a
    /// no-custom-skin version (reset to 1); `to` is what was written.
    Capped { path: String, from: u8, to: u8 },
    /// A `[players.NN]` whose target roster slot is empty; skipped.
    EmptySlot { slot: u8 },
}

impl TeamToml {
    /// Every field of the team and its roster's players, `Some`. `version` is the save's. An
    /// empty `players` gives a team/tactics-only document; a non-empty one missing a rostered
    /// id is `PlayerMissing` (a partial pool is a caller bug, a silent drop is data loss).
    pub fn from_team(version: PesVersion, team: &TeamEntry, players: &[&PlayerEntry]) -> Result<Self, TeamTomlError>;
    pub fn parse(text: &str) -> Result<Self, TeamTomlError>;
    /// The plan block's layout: one key per line, tables in the block's order, the range comments.
    pub fn to_toml(&self) -> Result<String, TeamTomlError>;
    /// Applies the `Some` fields onto `team` (id kept) and onto the players its roster slots
    /// name, all or nothing; `to` is the save's version, conversion happens when it differs from
    /// `pes_version`. Team TOML never adds players: an empty slot is a note.
    pub fn apply(&self, to: PesVersion, team: &mut TeamEntry, players: &mut [PlayerEntry]) -> Result<Vec<ImportNote>, TeamTomlError>;
}
/// `shirt_name_from`: the name upper-cased, colour codes stripped, cut to the version's shirt-name
/// field (the schema's text length minus the terminator) by characters.
pub fn shirt_name_from(name: &str, version: PesVersion) -> String;
```

Rules of `apply`: `pes_version` absent means "the target's version" (no conversion); the
ingame-face hex is written first (prefix-copied, `min(len)` bytes, the 2.17f rule), then the
`appearance` table through `settings_toml`'s shared writer; `base_copy_id` equal to the file's own player
id (the file's team id × 100 + slot) becomes the target player's id; a `stats` key the target
version lacks is `NotInThisVersion` when the versions differ and `TeamTomlError::NotInThisVersion`
when they are equal (a same-version file with a foreign key is a mistake, a converted one is
expected to shed keys); a skill the *source* version lacks is not written (the target's stands,
pair-wide, no note) and a skill the target lacks is dropped with a note; `name`/`shirt_name`
(and the team's) must fit the target version's text fields or `apply` is `TextTooLong` before
any write; the playing style goes
through `schema::playstyle::encode` for the target (2.17f's lists); instructions through
`schema::instruction::encode`; PES 15/16 targets drop the instructions table silently (the version
has none: a pair-wide fact). What `shirt_name_from` keeps of the character set beyond upper-casing
is unmeasured (the game's own edit screen restricts it); the function is the one place to
tighten when it is.

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

Both are raw dumps of the reference editor's in-memory structs; both parse into a `TeamToml`
(`interchange/legacy.rs`), so the import path is Team TOML's and there is one. Never written.

- **`.4ccs` squad files**: `"21a"` (3 ASCII bytes; the real files on this machine, written by an
  earlier build, carry `"20a"` with the same record — both tags are accepted, decision entry
  2.17h) + 2 ASCII digits of the exporting PES version + one **356-byte `player_export`
  record per rostered player**, in roster order + `[u16; 40]` shirt numbers (80 bytes) +
  optionally the 405-byte tactics block below (present when the file is 405 bytes longer than
  `5 + n × 356 + 80`; the editor writes it when tactics are enabled). The record is the MSVC
  layout of `player_export` (`editor.h`): little-endian, 4-byte alignment, `bool` one byte,
  `wchar_t[61]` UTF-16 name, `char[21]` shirt name, one padding byte at 271; the offsets are
  `scripts/provenance/fixtures/interchange_fixtures.py`'s ctypes table, the golden test holds
  them as literals. Fields the exporting version lacks are ignored whatever the bytes (the writer
  fills `phys_cont`/`clearing`/`reflex`/`cover` from `body_ctrl` on 15/16); the playing style is
  the exporting version's index. The record has no id and no ingame-face run: `ingame_face` is
  absent, skin/iris/player-gloves go to the `appearance` keys. Importable into any version
  through `TeamToml::apply` (the reference converts styles by version pair and clamps 19+ shirt
  numbers to 231 on older saves; `apply` instead refuses a number wider than the target's roster
  field before any write, like a too-long text, so a squad never applies and then fails to save).
- **`.4cct` "nightly" tactics files**: `"001"` + 2-char PES version + 8-char team id (ASCII,
  NUL-padded) + the **405-byte tactics block**: per preset, 3 × (11 position bytes, then 11 y/x
  pairs), 7 style bytes (attacking style, buildup, attacking zone, positioning, defensive style,
  containment area, pressure), 2 + 2 instructions each as canonical byte + player byte, support
  range, numbers in attack, defensive line, compactness, numbers in defence, fluid; then starting
  XI (11), bench (21), 6 set-piece takers (FK long, FK short, FK 2, CK left, CK right, PK), 3
  join-attack, 4 auto flags (substitution, offside trap, preset change, attack/defence levels) —
  **no captain**. The reference imports only into 16-18 and only from the same version (its
  x/y "conversion factor" is 1 for every pair); here the block parses into `TeamToml.tactics`
  and `apply`'s version rule decides, the header's version being `pes_version`. No real `.4cct`
  exists on this machine: the fixture is transcribed from the reference's write walk over the
  PES 19 fixture save's team 713 (provenance in the fixtures README), and the test's evidence is
  that the Rust reader's `TeamTactics` equals the schema codec's for that team.

### Texport (read + write)

Texport files are **PES's own in-game team export/import format**. Measured on real files
(three 17 texports, four 18, eleven 19, five 21; none for 15, 16 or 20): the file is a
concatenation of the **save's own records, in the save's own layouts**, so the schema codec
reads and writes every record and `interchange/texport.rs` adds only the crypto and the layout
table (`schema/texport.rs`). Two eras:

- **15-17** (`TEXPORT00000000`, ~5.6 MB): the version's savefile container (`SaveContainer`
  decrypts it unchanged; the description reads `Team Export Data NN` + the team name where a
  save's reads `Edit Data`). Tactics record at `0x10298` (15/16) / `0x10330` (17), player
  records at `0x51065C` / `0x510660` / `0x510840`, consecutive in roster order (15/16: player
  record then appearance record per player). The team and roster records' positions are not yet
  measured (the 17 payload holds the team id at `0x10054`, `0x10058`, `0x10234`, `0x1028C` before
  the tactics record); until they are, a 15-17 texport's team and roster come from the target
  save and only tactics and players are read. 15 and 16 offsets are the reference editor's,
  unverified.
- **18-21** (`.ted`, 8128 / 9648 / 14820 bytes for 18 / 19 / 21): plaintext header `0x00..0x30`,
  a 32-byte key at `0x30`, then the body XOR'd with the key starting at key index `0x12` / `0x13` /
  `0x14` / `0x15` (18 / 19 / 20 / 21) and wrapping at 32. PES 20 is unmeasured (no file exists):
  its key index is the reference editor's untested guess, its templates are PES 21's, and its
  size is the one its own record sizes derive (528-byte team, 244-byte roster: 14760 = `0x39A8`),
  not the `0x39E4` the reference assumes for both, since 21's size cannot hold 40 of 20's records
  after 20's shorter team record (decision, 2.17h-1). The body is `team record | 88-byte coach block |
  roster record | tactics record | 660-byte block | player records × roster width | 12-byte tail`,
  every size the save schema's (18: 480/164/628/188 × 32; 19: 416/244/628/188 × 40; 21:
  588/284/628/312 × 40), so tactics land at `0x32C` / `0x33C` / `0x410` and players at `0x834` /
  `0x844` / `0x918` as the reference reads them. The header is constant per version except the
  u32 at `0x0C` (two values seen on 18, one each on 19 and 21, independent of content) and the
  u32s at `0x10`/`0x14` (0 or 4): no content checksum was found (byte sums, CRC-32 and Adler-32
  of plaintext and ciphertext, with and without the key, match nothing). The coach block is
  `team id (u32) | 4 constant bytes | flag byte | "MANAGER" | zeros`; the 660-byte block is zero
  or a 125-188-byte run (an edited coach, presumably) and the tail is a per-version constant.

```rust
/// `schema/texport.rs`: the 18-21 layout, offsets derived from the record sizes.
pub struct TexportLayout {
    pub key_index: usize,       // first key byte used
    pub size: usize,            // whole file
    pub header: [u8; 0x30],     // the fixture's header, the writer's template
    pub coach: [u8; 88],        // the fixture's coach block minus the id, the writer's template
    pub tail: [u8; 12],
}
pub fn texport_layout(version: PesVersion) -> Option<&'static TexportLayout>;   // None on 15-17

/// `interchange/texport.rs`.
pub struct Texport { /* version, the plaintext (18-21) or the SaveContainer (15-17), team, players */ }
pub enum TexportError { UnknownVersion, WrongSize { version, expected, actual }, Container(ContainerError), Codec(CodecError), RosterMismatch { slot, roster: u32, player: u32 }, NoTemplate(PesVersion) }

impl Texport {
    /// `version = None` detects: by size for 18-21 (every size is distinct once 20's is derived;
    /// the candidate whose team, roster and tactics record ids agree wins, `UnknownVersion` when
    /// none or several do), then the container's version for 15-17. `Some(version)` on 15-17
    /// defers to the container's own version (no variant expresses the mismatch).
    pub fn from_bytes(bytes: &[u8], version: Option<PesVersion>) -> Result<Texport, TexportError>;
    /// A fresh 18-21 file from the layout's templates: header, coach (id patched), zero block,
    /// tail, a key from `salt[..32]`; `players` in roster order, ids checked against the roster
    /// (`RosterMismatch`). 15-17 is `NoTemplate`: write by reading a real file and replacing its
    /// team and players (`team_mut`/`players_mut`).
    pub fn new(version: PesVersion, team: &TeamEntry, players: &[&PlayerEntry], salt: &[u8; 320]) -> Result<Texport, TexportError>;
    pub fn version(&self) -> PesVersion;
    pub fn team(&self) -> &TeamEntry;            // team + roster + tactics (15-17: the tactics record's id,
                                                 //   the roster the consecutive records imply, no names)
    pub fn players(&self) -> &[PlayerEntry];     // roster order, empty slots skipped
    pub fn team_mut(&mut self) -> &mut TeamEntry;
    pub fn players_mut(&mut self) -> &mut [PlayerEntry];
    /// The records re-encoded into the plaintext (unmodeled bytes as read), then encrypted:
    /// XOR with the file's own key on 18-21, the container with `salt` on 15-17.
    pub fn to_bytes(&self, salt: &[u8; 320]) -> Result<Vec<u8>, TexportError>;
    /// The import document: `TeamToml::from_team` over the texport; on 15-17 the `[team]` section
    /// is reduced to `id` and shirt numbers are absent (the file carries neither).
    pub fn to_team_toml(&self) -> Result<TeamToml, TeamTomlError>;
}
```

Import never trusts the file's player IDs: players map onto the target team's roster by slot
order, which is `texport.to_team_toml()` then
`TeamToml::apply` — one import path for every format, conversion included (the reference zeroes
15/16 skills 28-41 and sets tight possession/aggression/physical contact to 77: the target's
values stand in for those instead, per the 2.17f rule). Because the game itself consumes these,
full compatibility matters in both directions: read (the reference already does) **and write**
(new — the reference is import-only), so the editor can produce a file PES can import. The
round trip `from_bytes` → `to_bytes` is byte-identical on every fixture (the codec and the
carried bytes cover the whole file); whether the game accepts a `new` file, and a file whose
records were edited, is the manual per-version check in [Verification](verification.md).

---

## Player section population

The savefile half of the [DB generator](../db_generator.md): from PES 19 the game writes a
fresh `EDIT00000000` from the database with the player count declared and **no player
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
only `id` and `base_copy_id` (the record's own id copy, `base_copy_id == id` meaning "unset");
`per_team` is capped at the version's roster width; the payload's player section grows to
`teams × per_team` records through `EditFile` — the one place player records are added, since
`to_bytes` otherwise keeps the file's own record count and order; on 15–18 the operation is
allowed but pointless (their fresh saves carry the players) and says so through a finding. The
PES 20 fixture is the day-0 save of a database these scripts generated, so populating a copy
with its player section stripped must reproduce it byte for byte.

---
