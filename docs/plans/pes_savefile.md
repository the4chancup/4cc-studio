# 4cc Studio — Savefile plan

Covers the `pes_savefile` lib crate: the EDIT00000000 file codec (container crypto,
schema-driven payload codec, player/team/tactics model), the cross-version player
data conversion, the save-to-save operations (aesthetics transplant, aesthetics
fingerprinting), and the interchange formats (Team TOML, legacy `.4ccs`/`.4cct`
readers, Texport). Shared by the [Save editor](save_editor.md), the
[Team compiler](team_compiler.md), the integrated
[model conversion](model_conversion.md), and the
[Export upgrader](export_upgrader.md). Platform context is in the
[core plan](core.md).

---

## Origin material

The crate consolidates savefile knowledge that today is spread across four codebases:

| Source | Location | Contributes |
|--------|----------|-------------|
| 4ccEditor | `Tools_4cc/4ccEditor-1` (an unofficial fork of the main `Tools_4cc/4ccEditor` with added functionality — strictly better despite being cloned first; use the fork as the reference) | Per-version payload schemas (`pes15.cpp`–`pes20.cpp`), player/team/tactics model (`editor.h`), bit engine (`data_util.cpp`), playstyle conversion tables (`menu_lists.cpp`), `.4ccs`/`.4cct`/Texport formats |
| Midcupping | `Tools_4cc/Midcupping` | Pure-Python reference implementations of the container crypto (PES 15's keyless scheme; PES 16/17 incl. their master keys), aesthetics block layout, transplant and aesthetics-diff logic |
| The converters | `Tools_4cc/4cc-aet-converter-19to16`, `Tools_4cc/aes_converter_16to21` | Container crypto for PES 16/19/21 (`save16.py`/`save19.py`/`save21.py`, incl. master keys), `convertPlayerSaveData` cross-version bitfield surgery |
| pesXdecrypter / libpesXcrypter | `Tools_4cc/pesXdecrypter` (the source of the DLLs; built copies in `Lab/4ccEditor/lib` and `Saves/4ccEditor_betaM/lib`) | All seven master keys (`src/masterkey.c`, incl. the PES 16 myClub variant) and the authoritative container spec (`src/crypt.c`) |

---

## Crate layout

The crate is organized in layers — bytes → fields → model → operations — and the layout exists to
keep per-version knowledge in exactly one layer. Everything version-specific (keys, container
shapes, bit offsets, payload section offsets) lives under `schema/` and `container/` as *data*; the
codec, the model, and every operation above them are version-generic and take a `&VersionSchema`.

```
crates/libs/pes_savefile/src/
├── lib.rs              # re-exports (PesVersion comes from the `pes_version` leaf crate, see libs.md)
├── file.rs             # EditFile: load (auto-detect) / save (.bak), retained unmodeled bytes
├── discovery.rs        # Documents\KONAMI layout per version → SavefileCandidate list
├── container/          # bytes ↔ decrypted payload
│   ├── mod.rs          #   Container trait, version auto-detection by key/shape trial
│   ├── pes16_21.rs     #   the shared 16–21 container (header, MT19937 stream, integrity)
│   ├── pes15.rs        #   the PES 15 LCG/MD5 chunk container
│   ├── mt19937.rs      #   keystream generator (known-answer tested)
│   └── keys.rs         #   the seven master keys, transcribed from masterkey.c
├── schema/             # per-version field tables — data only, no logic
│   ├── mod.rs          #   FieldSpec, VersionSchema, SectionLayout; schema_for(version)
│   ├── fields.rs       #   PlayerField / TeamField / TacticsField enums (the version-neutral vocabulary)
│   ├── pes15.rs        #   player + separate appearance tables, own bit-read variant flag
│   ├── pes16.rs        #   split player/appearance blocks
│   ├── pes17.rs        #   first unified block (+ phys_cont)
│   ├── pes18.rs        #   new container header, unified block
│   ├── pes19.rs        #   + star, 39 skills
│   └── pes20.rs        #   20/21 shared (+ mo_drib, tight_pos, aggres, play_attit, strong_hand; 41 skills)
├── codec/              # the one generic engine
│   ├── mod.rs          #   read_player / write_player / read_team / … over a &VersionSchema
│   └── bits.rs         #   bit-run reads/writes crossing byte boundaries (data_util.cpp's job)
├── model/              # what consumers edit
│   ├── mod.rs
│   ├── player.rs       #   PlayerEntry: abilities, skills, appearance, playstyles
│   ├── team.rs         #   TeamEntry: identity, roster, kit refs, colors
│   ├── tactics.rs      #   presets, formations, advanced instructions
│   └── names.rs        #   UTF-8 names, colour-code stripping (display_name)
├── settings_toml.rs    # PlayerSettings (the settings.toml model) ↔ PlayerEntry merge
├── convert.rs          # cross-version player conversion (bitfield surgery, playstyle/skill maps)
├── ops/                # save-to-save operations
│   ├── transplant.rs   #   aesthetics transplant (Midcupping)
│   ├── fingerprint.rs  #   aesthetics fingerprinting / diff
│   ├── compare.rs      #   comparator (gameplay + aesthetics)
│   └── fpc.rs          #   maps libs/fpc player presets onto PlayerEntry; interference check
└── interchange/        # text formats
    ├── team_toml.rs    #   Team TOML read/write (full fidelity)
    ├── legacy.rs       #   .4ccs / .4cct readers
    └── texport.rs      #   Texport read/write
```

Placement rules:

- **A bit offset appears in `schema/pesNN.rs` or nowhere.** No literal offsets in `codec/`, `model/`,
  or `ops/`; a new field is a `PlayerField` variant plus one table row per version that has it.
  This is what makes the roundtrip tests mechanically verifying.
- **`container/` knows nothing about players; `codec/` knows nothing about encryption.** `file.rs`
  is the only module that composes them.
- **`model/` types have no I/O and no version.** A `PlayerEntry` is the same struct for every
  version; fields a version lacks are `Option`/defaulted and the schema decides what is written.
- **`ops/` and `interchange/` consume `model/` only.** They never touch bytes; if one needs a
  version-specific fact, that fact is a schema query, not a match on `PesVersion`.
- **Reference constants keep their provenance in a doc comment** (which C++ file and function each
  table was transcribed from), so a mismatch found by a roundtrip test can be traced in one hop.
- Lib dependencies are `fpc` (the FPC system's data) and nothing format-specific; no Blender, egui,
  or tool dependency: this crate is consumed by the Team compiler, the Save
  editor, the Export upgrader, and the Player aesthetics editor, and must stay `wasm32`-checkable
  (core plan guardrail 6); discovery's filesystem walk is `cfg`-gated for non-desktop targets.

---

## Container format and crypto

**Correction to earlier analysis: the scheme is not AES.** The converters and
Midcupping show the full algorithm in pure Python: it is a custom stream cipher
whose keystream is generated by a **Mersenne Twister (MT19937)** with a
rotation/XOR whitening cascade. No AES anywhere; no `aes` crate needed. The whole
crypto layer is ~150 lines of Rust with zero dependencies beyond `md-5` (for PES15).

### PES 16–21 container

```
offset 0      320-byte encryption header ("salt") — random per file, encodes the file key
offset 320    file header, encrypted        (176 bytes on 16/17, 208 bytes on 18–21)
then          description ┐
              logo        │ four sections, each encrypted independently
              payload     │ (payload = the actual EDIT data the codec parses)
              serial      ┘
```

- **Effective master key**: the 64-byte per-version master key with bytes reversed
  within each 8-byte block: `eff[i] = key[(i & ~7) + 7 - (i & 7)]`.
- **File key recovery** (an involution — the same routine derives the key when
  writing a fresh random salt): `headerKey = eff ^ salt[256..320]`, decrypt
  `salt[0..256]` with it, then XOR the five 64-byte blocks of the result together.
- **Keystream** (`cryptStream`): seed MT19937 via the standard array-seeding
  routine (`init_by_array`) with the 64-byte section key as 16 little-endian u32;
  prime with four draws `c0..c3`; per output word: `c4 = next()`,
  `out = c4^c3^c2^c1^c0`, then rotate `c0=ror(c1,15)`, `c1=rol(c2,11)`,
  `c2=rol(c3,7)`, `c3=ror(c4,13)`. Data is XORed with the keystream.
- **Section keys**: `fileKey ^ (n as u64 LE)` where `n` = the header's byte length
  (176/208) for the header, then `0` description, `1` logo, `2` payload, `3` serial.
- **Header layout**: bytes 0–64 = the effective master key (this is the integrity
  check: wrong key or corrupt file fails the compare); bytes 64–80 = `payloadSize,
  logoSize, descriptionSize, serialSize` (4×u32; serial is stored halved — actual
  byte count is `serialSize * 2`); rest = identifier (on 18–21 this includes the
  game version string — 4ccEditor's `FileDescriptorNew` vs `FileDescriptorOld`).
- **Writing**: generate 320 fresh random salt bytes, derive the key from them,
  re-encrypt all sections. There are no checksums to fix up.
- This description is cross-verified against `crypt.c` (the source of the DLLs every tool
  loads): `cryptHeader`/`reverseLongs`/`xorRepeatingBlocks`/`cryptStream`/`xorWithLongParam`
  correspond function-for-function to the Python reference, so the Python scripts are faithful
  ports and the Rust port can be validated against either.

### PES 15 container

A different, keyless chunk format:

```
offset 0      1-byte cipher seed
offset 1      3 × 16-byte MD5 digests (description, logo, payload)
offset 49     description (384 bytes, fixed)
then          u32 logo length, logo
then          u32 payload length, payload
```

Cipher: a bytewise XOR stream driven by a 15-bit LCG state —
`c = (c*21 + 7) % 32768; byte ^= c % 255`, with the seed byte as `c₀`. Integrity: the three MD5 digests must match the decrypted
sections (recomputed on write; `md-5` crate).

### Master keys

64 bytes per version, embedded as constants. `src/masterkey.c` is the single authoritative
source for all of them (verified: the `MasterKeyPesNN` exports of both local `libpesXcrypter.dll`
copies match the source byte-for-byte, and `save19.py`/`save21.py`'s inline arrays match the
corresponding keys). The constants are the **raw** key — the per-8-byte reversal above is applied
by the consumer:

| Version | Key |
|---------|-----|
| 15 | none needed (keyless format) |
| 16 | `MasterKeyPes16` (plus `MasterKeyPes16MyClub` for the myClub edition, if that variant must auto-detect) |
| 17 | `MasterKeyPes17` |
| 18–21 | `MasterKeyPes18` … `MasterKeyPes21` |

### MT19937 note

The keystream MT19937 must match the reference implementation bit-for-bit,
including the `init_by_array` seeding. It is ~60 lines of Rust; port it directly
from the Midcupping scripts rather than trusting an external crate's seeding
behavior, and verify with known-answer tests against the Python output.

---

## Payload layout per version

Section offsets inside the decrypted payload, transcribed from 4ccEditor's
`DoFileOpen` and the Python scripts. PES15's player record is 0x70 (112) bytes:
`pes15.cpp`'s read walk ends there, and the unchanged-record write path skips 0x70.

| | Player count (u16) | Player block | Appearance block | Team IDs | Rosters | Tactics |
|---|---|---|---|---|---|---|
| PES 15 | 0x34 / 0x36 | 0x4C, 112 B | 0x2AB9CC, 68 B | 0x44AA6C | 0x4E45CC | 0x507194 |
| PES 16 | 0x34 / 0x36 | 0x4C, 112 B | 0x2AB9CC, 72 B | 0x46310C | 0x4FCC6C | 0x51F814 |
| PES 17 | 0x5C | 0x78, 188 B unified | ID +116, boots/gloves +120 | 0x3C3E58 | 0x475A90 | 0x490640 |
| PES 18 | 0x60 | 0x7C, 188 B unified | ID +116, boots/gloves +120 | 0x3C3E5C | 0x46FF54 | 0x488B74 |
| PES 19 | 0x60 | 0x7C, 188 B unified | ID +116, boots/gloves +120 | 0x5BCC7C | 0x6773C4 | 0x69EC8C |
| PES 20 | 0x60 | 0x7C, 312 B unified | ID +240, boots/gloves +244 | 0x8ED2FC | 0x9CCC04 | 0xA01E3C |
| PES 21 | 0x60 | 0x7C, 312 B unified | ID +240, boots/gloves +244 | 0x8ED2FC | 0x9D4648 | 0xA09880 |

Notes: on 15/16 the player-entry count is read at 0x34 (4ccEditor) and the
equal appearance-entry count at 0x36 (the Midcupping scripts). PES 18 shares
the 17/19 block layout (`pes18.cpp` walks the same 188 bytes); PES 20 shares
`pes20.cpp`'s 312-byte player walk with 21 but has its own roster/tactics
offsets — the two versions' section offsets match only for team IDs.

Structural eras:

- **PES 15/16**: the player block holds stats; a separate appearance array
  (68/72 bytes per player, keyed by player ID) holds boots/gloves IDs, physique,
  strip style, and the ingame face parameters.
- **PES 17–21**: one unified block per player; the appearance fields moved inside it as a
  72-byte sub-block whose **internal layout is identical across 16–21** (playerID, the
  boots/gloves u32, copy ID, physique bitfields, wrist/strip bitfields, the ingame-face run,
  skin color, iris, hair). Only its position moves: a separate array in 15/16, record +116 in
  17/18/19, record **+240** in 20/21. The 20/21 record grows to 312 bytes because the name
  section ahead of it widens (name 61 B at +54, shirt name 61 B at +115, print name 64 B at
  +176); within the sub-block: playerID +240, boots/gloves u32 +244, copy ID +248, physique
  +252, ingame-face run +262…+285, skin +285, iris +304, hair +304…+312. Verified empirically
  against a decrypted real PES 21 save (see Verification).
- ⚠ `convertTeam21.py` (the 16→21 reference) writes physique to record +128, the ingame-face
  run to +138…188, and the shirt name to +100 — stale PES 17/18/19 offsets — while its own
  240-anchored writes (boots/gloves u32 +244, wrist +259, skin/facial +285…) are correct. In
  the real layout those early offsets land in the name-section filler (`0xFE` padding and
  zeros), so the intended fields silently keep their template values and long names can be
  corrupted. The Rust port must take all 20/21 offsets from `pes20.cpp`'s walk, not from the
  converter (shirt name is at +115, physique at +252, ingame face at +262).
- Within the appearance data, the known anchor fields (from Midcupping and the
  converters): boots ID = 14 bits at bit offset 4 of the block's second u32,
  gloves ID = 14 bits at bit offset 18 of the same u32, face ID = the third u32
  (equal to the player ID when unset), physique bytes, strip-style bitfields,
  and a ~50-byte ingame-face parameter run.

---

## Schema-driven save codec

4ccEditor's six per-version codec files are near-duplicated sequential bitfield reads:

```cpp
players.lowpass  = read_data(0, 7, current_byte, pDescriptorNew);
players.loftpass = read_data(7, 7, current_byte, pDescriptorNew);
players.finish   = read_data(6, 7, current_byte, pDescriptorNew);
```

In Rust this becomes a declarative schema — one field table per version, one generic codec engine:

```rust
pub struct FieldSpec {
    pub field: PlayerField,
    pub bit_offset: u32,   // absolute bit position in the player block
    pub bit_width: u32,
}

pub struct VersionSchema {
    pub version: PesVersion,
    pub player_block_size: usize,
    pub fields: &'static [FieldSpec],
}

pub fn read_player(data: &[u8], schema: &VersionSchema) -> PlayerEntry;
pub fn write_player(player: &PlayerEntry, data: &mut [u8], schema: &VersionSchema);
```

~190KB of C++ collapses into static tables plus a ~100-line engine (the Rust
equivalent of `data_util.cpp`'s `read_data`/`write_data`/`read_data_raw`, which
handle bit runs crossing byte boundaries). This is also the most LLM-friendly
porting task in the suite: the LLM transcribes the C++ read sequences into table
entries, and roundtrip tests (read → write → compare bytes) verify correctness
mechanically.

Schema tables needed (following 4ccEditor's actual sharing):

| Schema | Source | Notes |
|--------|--------|-------|
| 15 | `pes15.cpp` | Own player + separate appearance schema, own bit-read variant |
| 16 | `pes16.cpp` | Split player/appearance blocks |
| 17 | `pes17.cpp` | First unified block; adds `phys_cont` |
| 18 | `pes18.cpp` | New container header; unified block |
| 19 | `pes19.cpp` | Adds `star`; 39 skills |
| 20/21 | `pes20.cpp` (shared, with 21-specific team functions) | Adds `mo_drib`, `tight_pos`, `aggres`, `play_attit`, `strong_hand`; 41 skills |

The same schema mechanism covers the team sections (team IDs, rosters, tactics),
not just player blocks.

### Whole-file API

The consumer-facing entry point wraps the container crypto and the schema codec:
`EditFile::load` (version auto-detected by trying master keys / container shape;
decrypt, parse) and `EditFile::save` (serialize, re-encrypt, and preserve the original in a `.bak`
before replacement — the shared convention behind the save editor's save action and the Team
compiler's savefile update stage). An open `EditFile` keeps its detected version; changing the
suite selector cannot reinterpret it. Compiler updates require a save matching the compile target.
Retain unmodeled container sections, payload bytes, padding, and unknown bits, patching only the
selected known fields; rebuilding only the modeled fields cannot satisfy lossless-save tests.

### Savefile discovery

The savefile does **not** live in the PES install folder — it is always under the user's
**Documents** folder, in a per-game KONAMI subfolder. Discovery therefore never looks at
`pes_folder_path`; it is a function of the PES version and the user profile, provided by this crate
(`discover_savefiles(version) -> Vec<SavefileCandidate>`) and used by the Team compiler's savefile
stage (`savefile_path = auto`) and the Save editor's Open dialog (initial folder / quick-open list).

Two layouts exist, verified on a live machine (PES 16, 17, 21 present; 15 confirmed to match 16,
18/19 reported to match 21 — fill the two game-folder names when a real install is available and
mark them verified):

| Versions | Path under `{Documents}\KONAMI\` | Notes |
|---|---|---|
| 15, 16, 17 | `Pro Evolution Soccer 20XX\save\EDIT00000000` | one save per game, no account level |
| 18, 19, 21 | `{game folder}\{account id}\save\EDIT00000000` | game folder: `eFootball PES 2021 SEASON UPDATE` (21); 18/19 names to verify (`PRO EVOLUTION SOCCER 2018` / `PRO EVOLUTION SOCCER 2019` expected, per the fixture notes under Verification). `{account id}` is an 18-digit numeric folder, one per account that has run the game on this profile |

Rules:

- **`{Documents}` is the shell's known folder, never `%USERPROFILE%\Documents`.** Documents is
  frequently relocated (the reference machine has it at `C:\Data\Documents`) or redirected by
  OneDrive; resolve it via `directories::UserDirs::document_dir()` (which wraps
  `SHGetKnownFolderPath(FOLDERID_Documents)`). On Linux/Wine the same relative layout applies under
  the Wine prefix's Documents; the KONAMI folder name is the anchor either way.
- **Account folders**: with exactly one `{account id}` folder containing a `save\EDIT00000000`, that
  is the result. With several, all are returned as candidates ordered by the savefile's modification
  time, newest first; `auto` mode takes the newest and reports which one it chose
  (`savefile_autodetected`, Info, with the path) so a user with two accounts can pin `savefile_path`
  if the guess is wrong. Folders without a savefile are ignored, not errors.
- **The version is verified after discovery**, not assumed from the folder: the candidate is loaded
  with the ordinary `EditFile::load` auto-detection and must match the requested version, otherwise
  it is reported as a mismatch rather than used (a stale save left by a previous game version in a
  reused folder must not be edited as if it were current).
- **Nothing found** → the consumer's own message (`savefile_missing` in the compiler; an empty
  quick-open list plus the ordinary file picker in the editor). Discovery never creates folders.

The per-version table is data (`phf` map keyed by version), like the skeleton and game-path tables,
so filling in the 18/19 folder names is a one-line change.

---

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
0x00–0x0C; 18+: 0x00–0x0F, different meanings), and **playstyle enums are
version-specific** — 4ccEditor's `menu_lists.cpp` carries twelve conversion
arrays (16↔17/18, 16↔19, 16↔20/21, 17/18↔19, 17/18↔20/21, 19↔20/21). In Rust
these become one canonical `PlayStyle` enum plus per-version encode/decode maps,
which is what makes cross-version team import work.

---

## Player settings model (settings.toml)

The crate owns the `PlayerSettings` model behind the export format's
`settings.toml` files (see "Player settings in exports" in the
[Aesthetics export plan](aesthetics_export.md)): parsing the TOML, applying it to a
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
`display_name()` helper that strips the codes (fixed-length match — the hex run
is exactly 8, so a name letter `a`–`f` following a code is never eaten); the
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
the [Save editor plan](save_editor.md)). Both the save editor's FPC toggle and
the Team compiler's `fpc.on`/`fpc.off` marker files use the same preset values. The compiler
composes the boots/gloves ID fields with asset outcomes: a successful requested standalone output
uses its assigned ID, a failed requested output preserves the existing save ID, and no requested
standalone output allows the preset ID. Other preset fields still apply normally; the full
precedence rule lives in the [Aesthetics export plan](aesthetics_export.md)'s "FPC toggle" section.

---

## Cross-version player conversion

The converters' `convertPlayerSaveData` becomes a `convert` module: rewrite a
player's appearance data from one version's layout to another's, preserving what
translates and capping what doesn't. Used by the integrated
[model conversion](model_conversion.md) (when compiling an export against a
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

---

## Save-to-save operations

Ported from Midcupping, exposed in the [save editor](save_editor.md):

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
  (see "Savefile update" in the [Team compiler plan](team_compiler.md)). One path, so a patch applied
  a week later by someone else yields the same bytes the DLC builder's machine wrote.
- `pes_version` must equal the save's; there is no cross-version application — the compile targeted
  a version and its IDs and presets are that version's. `allocation_scheme_version` must equal the
  suite's; a patch from an older scheme is refused with the pointer to the from-scratch remake rule.
  A team ID absent from the save skips that team with an error and applies the rest; referees are
  team 999 like anywhere else.
- Applying is the only aesthetics write path the save editor leaves open by default (see "Read-only
  aesthetics" in the [Save editor plan](save_editor.md)); it goes through the editor's undo pipeline
  and the comparator's preview like any other bulk edit.

### Legacy 4ccEditor formats (read-only)

- **`.4ccs` squad files** (version tag "21a"): binary `player_export` array +
  optional tactics. Readable and importable; never written.
- **`.4cct` "nightly" tactics files** (version tag "001"): binary tactics dump.
  Readable and importable; never written. Import targets PES 16–18 only (4ccEditor's
  own gate — later saves go through Texport instead), and the file header records
  the exporting PES version, which the import must match exactly.

Both parse into the same model structs; the write path is Team TOML only.

### Texport (read + write)

Texport files are **PES's own in-game team export/import format** — same
container crypto as the savefile, different internal layout (tactics first, then
player entries, per-version offsets; 4ccEditor's `handle_texport` +
`fill_player_entryXX_texport`/`fill_team_tacticsXX_texport` are the reference).
Because the game itself consumes these, full compatibility matters in both
directions: read (4ccEditor already does) **and write** (new — 4ccEditor is
import-only), so the editor can produce a file PES can import. Write support
needs the full texport layout, not just the fields 4ccEditor reads; transcribe it
per version and verify by importing the output into the game. If full-layout
transcription proves impractical for some version, the fallback is
template-patching: load a real texport of that version and rewrite the fields we
model.

---

## Verification

- **Container roundtrips**: decrypt → re-encrypt → decrypt again must reproduce
  the payload byte-for-byte for every version (encryption uses a random salt, so
  ciphertexts differ; plaintexts must not). Known-answer tests for the MT19937
  keystream against the Python reference.
- **Codec roundtrips**: read → write → byte-compare for every player and team
  block in real saves of all seven versions (`Midcupping/EDIT.bin` and friends
  are test fixtures; the user's KONAMI `eFootball PES 2021 SEASON UPDATE` save
  is the 20/21 fixture — sanity anchors already verified there: player count
  u16 at 0x60, and every record's appearance playerID at +240 equals its player
  ID at +0).
- **Name fields**: names read as null-terminated UTF-8 (the `♂` in a real PES 19
  name is the bytes `E2 99 82`, colour codes are the ASCII bytes `11 63` + hex)
  from real PES 19 (`PRO EVOLUTION SOCCER 2019` KONAMI save) and PES 21 cup
  saves through the AET compiler Red's `savefile.py` reader, which shares this
  plan's offsets; `display_name()` must strip every colour code in those saves'
  ~30 decorated names and leave every undecorated name unchanged.
- **Export settings versus save interchange**: generated `settings.toml` omits boots/gloves IDs,
  and authored ID keys are rejected; full Team TOML still round-trips those IDs as player-record
  data. Compiler ID assignment remains independent of the authorable settings serializer.
- **Settings completeness**: for every version, each appearance field in the schema table is
  marked `settings` or `compiler_owned`, and the `settings` set equals `PlayerSettings`' fields;
  the generated template contains every `PlayerSettings` key (a commented line for each unset one).
- **Patch equivalence**: compiling a fixture export against a save, and compiling it without a save
  then applying the produced patch to a copy of that save, yield byte-identical savefiles; applying
  a second patch covering a subset of teams changes only those teams' players and only the fields
  it carries; a patch with a different `pes_version` or an older `allocation_scheme_version` is
  refused without touching the save.
- **Cross-implementation parity**: field values must match 4ccEditor's display
  for the same save; aesthetics fingerprints must match `compare-saves-*.py`
  output; transplant output must byte-match `transplant-aesthetics-*.py`.
- **Cross-version conversion**: compare against the converters' verified behavior on real team
  data, excluding their documented stale PES20/21 offsets. Assert corrected name and appearance
  fields independently against the target schema; reproducing the known converter corruption is
  not parity.
- **Texport write**: import the generated file in the actual game (manual, per
  version).
