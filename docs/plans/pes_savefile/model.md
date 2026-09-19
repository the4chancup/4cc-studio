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

`PlayerSettings` covers **every field of the savefile's player appearance record** — physique,
strip style (taping, spectacles and their colour, sleeves, inners, socks, undershorts, shirttail,
gloves, winter gloves), wrist-tape colours, skin and iris colour, the motion block (hunching, arm
movement, kick motions, gc1/gc2, dribbling motion on 20+; visual choices a team makes, so
aesthetics), and the ingame-face parameter block (facial feature types, colours — the data
Midcupping fingerprints) — with exactly two exclusions, both **compiler-owned**: the boots/gloves
model IDs (derived from the models and links present) and the edit flags (derived from what was
written — face/hair/physique/strip bits, base-copy ID). Export `settings.toml` neither accepts the
excluded keys nor emits them when generated from a save. Completeness is a tested invariant, not a
convention: the per-version schema tables mark each appearance field as `settings` or
`compiler_owned`, and a test asserts that every appearance field carries one of the two marks and
that the `settings` set equals `PlayerSettings`' fields for that version — a field added to a schema
cannot be silently left out of the authorable set. The generated template (the Export upgrader's
and the save editor's `settings.toml` output, and the blank template for a new player folder)
lists **every** `PlayerSettings` key: set ones with their value, unset ones as commented lines
carrying the key's range in the injected comment, so the file itself shows what can be authored
and "absent = untouched" stays true for the unset ones. This matters because the patch workflow
below makes `settings.toml` the *only* route by which a team's aesthetics reach the official save:
a field the file cannot express is a field nobody can set.

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
