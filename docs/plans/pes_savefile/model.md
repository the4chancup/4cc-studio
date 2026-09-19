# 4cc Studio — Savefile plan: Data model

Part of the [Savefile plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player/team/tactics model

One version-independent model, populated by whichever schema loaded the file.
4ccEditor's ~100-line manual `operator==` and `PlayerExport()`/`PlayerImport()`
copy methods become derived `PartialEq`/`Clone`/serde.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerEntry {
    pub id: u32,
    pub name: String,          // on disk: null-terminated UTF-8, 46 B (17–19) / 61 B (20/21);
                               // 4ccEditor's wchar[61] is only its in-memory buffer
    pub shirt_name: String,    // on disk: null-terminated ASCII, 18 B (17–19) / 61 B field (20/21,
                               // 21 read); 4ccEditor's char[21]
    pub basic: PlayerBasics,        // nation, age, height, weight, shirt number
    pub stats: PlayerStats,         // ~25 ability values + form, injury, weak foot
    pub positions: PlayerPositions, // registered position, play_pos[13] (A/B/C), playstyle
    pub skills: PlayerSkills,       // play_skill[..41], com_style[7]
    pub motion: PlayerMotion,       // hunching, arm movement, kick motions, gc1/gc2
    pub edit_flags: PlayerEditFlags,// b_edit_face/hair/phys/strip/player/…, base copy
    pub appearance: PlayerAppearance, // boots/gloves IDs, physique, strip style,
                                      // colors, ingame face parameters
    // Version-gated fields:
    pub star: Option<u8>,             // 19+
    pub tight_possession: Option<u8>, // 20+
    // ...
}
```

Version-gated fields become `Option<T>`:

| Fields | Versions |
|--------|----------|
| `catching` ("Saving") | 15 only (replaced by clearing/reflex/coverage) |
| `clearing`, `reflex`, `coverage` | 16+ |
| `phys_cont` | 17+ |
| `star` | 19+ |
| `mo_drib`, `tight_pos`, `aggres`, `play_attit`, `strong_hand` | 20+ |
| available skills | 22 (15, mapped into 28 canonical slots), 28 (16–18), 39 (19), 41 (20/21) |

The model favors readable field names — e.g. `tight_possession` where the schema
tables quote 4ccEditor's source identifier `tight_pos` verbatim; the schema layer
maps between the two spellings.

The team side is a full model, not just names — the save editor needs all of it:

```rust
pub struct TeamEntry {
    pub id: u32,
    pub name: String,            // on disk: null-terminated UTF-8, 0x46 B (4ccEditor's wchar[0x46]
                                 // is its in-memory buffer)
    pub short_name: String,      // 3 chars
    pub manager_id: u32,
    pub stadium_id: u32,
    pub colors: [Rgb; 2],
    pub roster: Vec<RosterSlot>, // up to 40: player ID + shirt number
    pub starting_eleven: [u8; 11],
    pub bench_order: Vec<u8>,
    pub kit_slots: Vec<StripSlot>,   // stripBlock[10]: kit number ↔ team ID binding
    pub set_pieces: SetPieceTakers,  // FK long/short/2, CK L/R, PK, captain
    pub auto_flags: TeamAutoFlags,   // auto sub, offside trap, atk/def levels, preset change
    pub presets: [TacticsPreset; 3],
}

pub struct TacticsPreset {
    pub style: TacticStyle,          // attacking/defensive style, zones, buildup,
                                     // positioning, containment, pressure, fluid
    pub sliders: TacticSliders,      // support range, defensive line, compactness (1–10),
                                     // numbers in attack/defense (1–3)
    pub formations: [Formation; 3],  // kick-off, in-possession, out-of-possession
    pub attack_instructions: [AdvancedInstruction; 2],
    pub defense_instructions: [AdvancedInstruction; 2],
}

pub struct Formation {
    pub players: [FormationSlot; 11], // x, y, position (GK..CF)
}
```

Version quirks to encode: advanced-instruction value ranges differ (PES 17:
0x00–0x0C; 18+: 0x00–0x0F, different meanings — 4ccEditor resolves this through
a canonical 17-order enum: values 0x08–0x0C shift +2 for 18+, and 18+'s
Defensive/False Winger/Wingback are 0x0D/0x0E/0x0F in canonical order, mapping
to 0x08/0x09/0x0F), and **playstyle enums are
version-specific** — 4ccEditor's `menu_lists.cpp` carries twelve conversion
arrays (16↔17/18, 16↔19, 16↔20/21, 17/18↔19, 17/18↔20/21, 19↔20/21). In Rust
these become one canonical `PlayStyle` enum plus per-version encode/decode maps,
which is what makes cross-version team import work.

---

## Player settings model (settings.toml)

The crate owns the `PlayerSettings` model behind the export format's
`settings.toml` files (see "Player settings in exports" in the
[Aesthetics export plan](../aesthetics_export/README.md)): parsing the TOML, applying it to a
`PlayerEntry`, and the reverse — generating `settings.toml` from an existing
savefile (used by the save editor's migration feature and the Export upgrader).
Keeping both directions next to the codec ensures the TOML schema and the
savefile fields cannot drift apart.

**Every field is optional**: applying a `PlayerSettings` only touches the
savefile fields it actually specifies (`Option<T>` throughout; `None` = leave
the savefile value alone). This includes the player name — name writing is
opt-in (`name = true` derives it from the folder, a string sets it explicitly,
absent means untouched); the derivation rules are the Team compiler's, and this
crate just applies whatever the resolved `PlayerSettings` carries.

**Name colour codes.** Cup savefiles colour player names in-game with an inline
code: the byte `0x11`, the ASCII letter `c`, then **8 hex digits** (RRGGBB plus
an alpha byte, `ff` in every observed case) — `\x11ce5de00ffXIAO MEI MEI`. Codes
may appear anywhere in the name, several times, to colour segments differently
(`PEP\x11cffffffffSI \x11c004b93ffMAN` renders "PEPSI MAN" in three colours).
They are used freely — medal players are the common case, but not the only one.
Verified against ~30 decorated names across real PES 19 and 21 cup saves (see
Verification). No tool in the 4cc toolset writes them; they are hand-edited by
cup organizers and must be **preserved byte-for-byte** by every copy-through,
transplant and roundtrip path. This crate exposes the raw name and a
`display_name()` helper that strips the codes (fixed-length match: the colour is exactly the 8
bytes after `\x11c`, hex or not, so a name letter `a`–`f` following a code is never eaten; the
2-byte reset `\x11d` is stripped too; see `codec.md` "Whole-file API" for the measurement); the
save editor renders the colours from the raw name and allows editing them, while
folder-name derivation (Team compiler, Export upgrader) and search use the
stripped form. Any other control character in a name is not a known code and is
left alone.

`PlayerSettings` covers **every decoded field of the savefile's player appearance record** —
physique, strip style (taping, spectacles and their colour, sleeves, inners, socks, undershorts,
shirttail, player gloves and their colour), wrist-tape colours, skin and iris colour, the motion
block (hunching, arm movement, kick motions, gc1/gc2, dribbling motion on 20+; visual choices a
team makes, so aesthetics), and the eleven ingame-face feature types — with exactly two exclusions,
both **compiler-owned**: the boots/gloves model IDs (derived from the models and links present)
and the edit flags plus base-copy ID (derived from what was written). Export `settings.toml`
neither accepts the excluded keys nor emits them when generated from a save.

**The ingame-face run.** From its 22nd byte to its end, the appearance block (72 bytes on PES 16 to
21, 68 on PES 15; a separate record on 15/16, inside the player record from 17) is a run no legacy
tool decodes as a whole: the reference editor reads only the player-gloves bits, skin and iris out
of it, the two converters name eleven feature types by bit position and cap them, and the
transplant and fingerprint scripts copy and hash it as bytes. Reverse-engineering the rest means a
running game per version with a save diff per slider, so the crate models the run as **opaque
bytes with typed accessors**: `PlayerAppearance.ingame_face: IngameFace`, a `Vec<u8>` of the run
(50 or 46 bytes; the schema says which) carried verbatim, and one field table for the bits inside
it that are known, identical on every version because the block's internal layout is (measured
in `container.md`):

```rust
/// A known bit run inside the ingame-face run; the fifteen the legacy tools read or write.
/// Offsets are relative to the run's first byte (byte 22 of the appearance block).
pub enum IngameFaceField {
    PlayerGloves,       // byte 0 bit 0, 1 bit
    PlayerGlovesColor,  // byte 0 bit 1, 3 bits
    SkinColor,          // byte 23 bit 0, 3 bits
    CheekType,          // byte 23 bit 3, 5 bits
    ForeheadType,       // byte 24 bit 0, 3 bits
    FacialHairType,     // byte 24 bit 3, 5 bits
    LaughterLinesType,  // byte 25 bit 0, 3 bits
    UpperEyelidType,    // byte 25 bit 3, 3 bits
    LowerEyelidType,    // byte 26 bit 0, 3 bits
    EyebrowType,        // byte 28 bit 0, 3 bits
    NeckLineType,       // byte 28 bit 5, 2 bits
    NoseType,           // byte 30 bit 0, 3 bits
    UpperLipType,       // byte 31 bit 0, 3 bits
    LowerLipType,       // byte 31 bit 3, 3 bits
    IrisColor,          // byte 42 bit 0, 4 bits
}

/// `schema/ingame_face.rs`: the table above as `FieldSpec<IngameFaceField>` rows (the one
/// place these offsets appear), plus the run's place in each version's record.
pub const INGAME_FACE_FIELDS: &[FieldSpec<IngameFaceField>];

/// Where the run sits in a record that carries it.
pub struct ByteRun { pub byte_offset: u32, pub len: u32 }
// `RecordSchema` gains `pub ingame_face: Option<ByteRun>`: `Some` on the PES 15/16 appearance
// record and the PES 17+ player record, `None` on every other record (15/16 player, team, roster).

/// `model/ingame_face.rs`
pub struct IngameFace(Vec<u8>);
impl IngameFace {
    pub fn get(&self, field: IngameFaceField) -> u8;
    /// `CodecError::ValueTooWide` when the value does not fit the field's bits.
    pub fn set(&mut self, field: IngameFaceField, value: u8) -> Result<(), CodecError>;
    pub fn bytes(&self) -> &[u8];
    /// The run with the player-gloves and skin bits zeroed: what the fingerprint hashes
    /// (the reference masks the same two, because they overlap fields it lists separately).
    pub fn normalized(&self) -> Vec<u8>;
}
```

The four fields the tables used to list as bit runs (`SkinColor`, `IrisColor`, `PlayerGloves`,
`PlayerGlovesColor`) leave the per-version tables and the `PlayerField` enum: they are reached
through `IngameFace::get`/`set`, so no byte has two owners and the schema overlap test keeps it
that way. `read_player_into` copies the run out of the record; `write_player` copies it back and
rejects a run of the wrong length (`CodecError::RunSize`). The generator emits the `ingame_face`
entry and omits the four fields; the round-trip tests over every real payload are unchanged and
still prove byte identity. Cross-version conversion (`convert.rs`) is where the 46-byte PES 15
run meets the 50-byte one; that step decides the padding rule.

What this gives up is written down rather than papered over: the plan's earlier "every field"
becomes "every decoded field", the undecoded bits (hair, slider positions, any colour beyond skin
and iris) are carried but not authorable, and the *evidence* for the eleven types is the two
converters' agreement on positions and caps, not a game session. Decoding more of the run later
is one row in `INGAME_FACE_FIELDS` plus one `settings.toml` key; nothing else moves.

**Completeness is a tested invariant, not a convention.** `settings_toml.rs` classifies every
`PlayerField` and every `IngameFaceField` exhaustively (an `Ownership::{Settings, CompilerOwned,
Gameplay}` match with no wildcard arm, so a new field fails to compile until classified), and one
test asserts that the `Settings` set equals the set of fields the `SettingKey` table reaches —
a field added to a schema cannot be silently left out of the authorable set. The generated
template (the Export upgrader's and the save editor's `settings.toml` output, and the blank
template for a new player folder) lists **every** `SettingKey`, one per line, each followed by the
comment carrying its range: set ones with their value, unset ones as commented lines, so the file
itself shows what can be authored and "absent = untouched" stays true for the unset ones. The key
table, its order, its comments and its ranges are the block in the
[Aesthetics export plan](../aesthetics_export/settings_toml.md) ("Player settings in exports"),
which the code reproduces literally. This matters because the patch workflow below makes
`settings.toml` the *only* route by which a team's aesthetics reach the official save: a field the
file cannot express is a field nobody can set.

```rust
/// `settings_toml.rs`
pub enum NameSetting { FromFolder, Explicit(String) }

/// One TOML key of the table; the enum is the table's row set and the `spec` function its
/// columns (table path, key name, kind, widest range, range comment). Physique keys convert
/// between the game number and the stored value (+7); 1-based motion keys between the game
/// number and the stored value (-1); string-enum keys between the listed labels and the stored
/// index.
pub enum SettingKey { SkinColor, IrisColor, Height, /* … every key of the block, in its order */ LowerLipType }

/// Every leaf is `Option`; `None` = leave the savefile value alone. The struct mirrors the
/// TOML tables (`appearance`, `appearance.physique`, `.strip`, `.motion`, `.face`) with the
/// model's storage types (`u8`/`bool`), not the TOML surface types: the string labels and the
/// signed physique numbers exist only at the parse/emit boundary.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerSettings { pub name: Option<NameSetting>, pub appearance: AppearanceSettings }

impl PlayerSettings {
    /// Parses a file: unknown keys and tables, wrong types, out-of-range values and unknown
    /// labels are `SettingsError`s naming the key; the compiler-owned keys are unknown keys.
    pub fn parse(text: &str) -> Result<Self, SettingsError>;
    /// Every key `Some` from the player; `name` is `Explicit(raw name)`, colour codes included.
    pub fn from_player(player: &PlayerEntry) -> Self;
    /// Writes the `Some` fields onto the player. `NameSetting::FromFolder` is not applied: the
    /// caller (the Team compiler) resolves it to `Explicit` first, since the folder rules are
    /// its. A `Some` on a field this player's version lacks (`dribbling` before PES 20) is
    /// `SettingsError::NotInThisVersion`.
    pub fn apply(&self, player: &mut PlayerEntry) -> Result<(), SettingsError>;
    /// The complete template: every key in table order, set ones as values, unset ones
    /// commented, each with its range comment.
    pub fn to_toml(&self) -> String;
    /// Edits an existing document in place (`toml_edit`, comments preserved), setting the
    /// `Some` keys and leaving everything else as the user wrote it.
    pub fn update_toml(&self, document: &mut toml_edit::DocumentMut);
}
```

`ops/fpc.rs` maps the `libs/fpc` presets onto a player and derives the interference inputs:

```rust
/// Writes one preset's appearance (sleeves, tuck, socks, boots/gloves IDs, skin). The caller
/// substitutes a custom model's own boots/gloves IDs into `appearance` first (the compiler's
/// precedence table). `SkinColor::Custom` writes skin 7 on versions that have a custom skin
/// (`fpc::custom_skin_available`) and is `FpcError::CustomSkinUnavailable` elsewhere;
/// `SkinColor::Preset` resets a skin of 7 to 1 (light) and leaves any other skin alone.
pub fn apply(player: &mut PlayerEntry, appearance: &fpc::Appearance, version: PesVersion)
    -> Result<(), FpcError>;
/// The player's strip settings as the interference inputs.
pub fn strip_style(player: &PlayerEntry) -> fpc::StripStyle;
/// On a hide or partial-hide preset by the 4cc convention: the nonexistent boots ID, or a
/// custom skin.
pub fn is_fpc_player(player: &PlayerEntry) -> bool;
```

The authorable settings subset shares one schema between export `settings.toml` and Team TOML.
Full-fidelity Team TOML additionally preserves the save's boots/gloves IDs as player-record data
outside that subset; generating an aesthetic export must not copy those IDs into `settings.toml`.

The crate also owns the **version-aware FPC enable/disable presets** (Full
Player Customization invisibility — nonexistent boots/gloves IDs plus strip
settings, with small per-version differences like the pre-18 custom skin; see
the [Save editor plan](../save_editor.md)). Both the save editor's FPC toggle and
the Team compiler's `fpc.on`/`fpc.off` marker files use the same preset values. The compiler
composes the boots/gloves ID fields with asset outcomes: a successful requested standalone output
uses its assigned ID, a failed requested output preserves the existing save ID, and no requested
standalone output allows the preset ID. Other preset fields still apply normally; the full
precedence rule lives in the [Aesthetics export plan](../aesthetics_export/fpc_toggle.md)'s "FPC toggle" section.

---
