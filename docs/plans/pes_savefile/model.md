# 4cc Studio — Savefile plan: Data model

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Player settings model (settings.toml)

What exists of this is in `spec/pes_savefile/model.md` "Player settings model (settings.toml)".

**Planned for Phase 5**, landed in the crate except its consumers (decision entry "aesthetics
travel in the database tables"), which stay open:

- The Team compiler resolves `NameSetting::FromFolder` (`name = true` in a player's file) to
  `Explicit` by its folder rules before `apply`; this crate applies whatever the resolved
  `PlayerSettings` carries.
- **The Fox database row.** The compile writes the rows `appearance_row` produces for a
  stripped player into its `PlayerAppearance.bin`; boots and gloves go to the
  `BootsList.bin`/`GloveList.bin` pairs, not to the row.
- **A save player's row from his record**, for the Save editor's strip and seed:
  `EditFile::appearance_row(player_id) -> Result<[u8; 60], SaveError>`, the player id and the
  56 bytes his record holds after the base-copy id, his entry as the save would write it, so a
  value past a key's range is carried as stored, where the settings route
  (`PlayerSettings::appearance_row`, still the route for a settings file) refuses it. The 56
  bytes are all modeled fields (packed fields, then the ingame-face run kept verbatim; found at
  5.4c1a), so the entry is written over a zeroed record: a copy of the save's record changed
  nothing (its mutation run's two survivors said so), and the test comparing rows with the
  payload is what fails if a schema ever leaves a gap there.
  Tested on
  each Fox fixture save: an unedited player's row is his record's bytes in the loaded payload,
  and an edited field shows in it. A pre-Fox save has no row (an error naming the version).
- **The Fox player tables** live here too, as one type: `PlayerAppearance.bin` (60-byte rows,
  the player id then the 56 appearance bytes), `BootsList.bin` and `GloveList.bin` ((player
  id, item ID) `u32` pairs), each read whole (a length that is not a whole number of rows is
  an error naming the table), edited by player id (set a row, remove a row) and written sorted
  by player id. Every table the games and the cup's DLC ship is sorted with each id once (12
  `PlayerAppearance.bin` files of PES 17 to 20 and the cup's `4cc_01`, measured 2026-10-14).
  The Team compiler's bins step (its `BootsList`/`GloveList` editing moves here from
  `team_compiler::bins::player_tables`, its row-selection rules staying in the tool) and the
  Save editor's strip and seed edit the same tables, and a tool crate cannot call another's
  code; a third copy of a table codec is what this avoids.
- Generation's callers skip a record whose appearance id is -1 (`settings_toml.md`, the end
  of "Player settings in exports"); `from_player` itself reads whatever record it is given.
- Only the Save editor's strip operation writes the appearance id's -1 (worklog issue
  "player id versus appearance id").

**The ingame-face run, decoded (before 1.0.0).** The spec models the run as opaque bytes with
fifteen named fields, because no legacy tool or published source decodes more
(`spec/pes_savefile/model.md` "The ingame-face run"). Every other bit has since been measured in
the games (2026-10-13). In Edit mode each player of one team had one setting changed, then the
team was saved once, so the diff per player attributed each bit run to its setting. The rest was
measured from renders of patched saves, and from censuses of the games' own face tables (PES 17
to 21) and of every cup save (PES 15 to 21). PES 21 and PES 15 were measured whole; the renders
were checked on PES 17 and 19. So the run has no undecoded bit left: each bit is a field below,
or measured unused.

**Two layouts.** PES 16 to 21 share one layout of the 50-byte run. PES 15's 46-byte run shares
its face part, its colours and its accessory bits, and differs at four places:

- byte 34 holds the hair variation, where 16 to 21 hold the hairstyle and cropped style;
- byte 38 bits 3 to 5 hold the hair type, where 16 to 21 hold the wave level;
- bytes 42 bit 4 to 45 are unused, where 16 to 21 hold the rest of the hairstyle;
- the nose field codes the nose angle with the nose type.

It also has no bytes 46 to 49. Every list field is as wide as PES 15 uses it, which is wider than
the ranges of 16 to 21 need. Each extra high bit is 0 in every face the games or the cups made on
16 to 21. Set, the game draws past the models it ships, and PES 21 crashes on a set high bit of
the cropped style. So the ranges of 16 to 21 keep these bits 0.

The fields, as run byte:bit and width in bits, in `settings.toml`'s `[appearance.face]` unless
said. Offsets are relative to the run's first byte, as in the spec's table:

- **Sliders.** 4 bits each; the key is -7 to 7, stored plus 7. They exist on every version:
  - face: `jawline` 0:4, `jaw_height` 1:0, `cheekbones` 2:0, `cheekbone_height` 2:4,
    `chin_width` 21:4, `chin_height` 22:0, `chin_depth` 22:4, `face_size` 29:0,
    `face_height` 29:4;
  - eyebrows: `brow_width` 4:4, `inner_eyebrow_height` 5:0, `eyebrow_depth` 5:4,
    `temple_width` 6:0, `outer_eyebrow_height` 6:4;
  - eyes: `pupil_size` 7:4, `eye_depth` 8:0, `inner_eye_position` 8:4, `inner_eye_height` 9:0,
    `outer_eye_position` 9:4, `eye_corner_height` 10:0, `upper_eyelid_width_inner` 10:4,
    `upper_eyelid_height_inner` 11:0, `upper_eyelid_width_outer` 11:4,
    `upper_eyelid_height_outer` 12:0, `bottom_eyelid_height` 13:0, `horizontal_eye_position`
    13:4, `eye_height` 14:0;
  - nose: `nose_height` 14:4, `nose_tip_depth` 15:0, `nostril_width` 15:4, `nose_depth` 16:4,
    `nose_width` 17:0;
  - mouth: `mouth_height` 18:0, `mouth_depth` 18:4, `lip_size` 19:0, `mouth_corner_height` 20:0,
    `lip_width` 20:4;
  - ears: `ear_length` 26:4, `ear_width` 27:0, `ear_angle` 27:4.

  The names are PES 21's labels. `cheekbone_height` is PES 15's slider; 16 to 21 have no
  slider for it but still draw it.
- **Sliders only PES 15 draws.** These are stored on every version, and the games' tables fill
  them for every face. The renders of 16 to 21 ignore them. Their editor writes the first three
  with their main slider's value and keeps the others:
  - `pes15_jaw_depth` 1:4;
  - `pes15_cheekbone_depth` 3:0;
  - `pes15_mouth_edge_position` 21:0;
  - `pes15_eyebrow_depth` 7:0;
  - `pes15_bottom_eyelid_position` 12:4;
  - `pes15_nasal_bridge_position` 16:0;
  - `pes15_nasal_root_height` 17:4.
- **Feature types.** These are the existing eleven keys, stored as is, with their fields
  widened as above:
  - `cheek_type` 23:3 w5;
  - `forehead_type` 24:0 w3;
  - `facial_hair_type` 24:3 w5;
  - `laughter_lines_type` 25:0 w3;
  - `upper_eyelid_type` 25:3 w5 (was 3);
  - `lower_eyelid_type` 26:0 w4 (was 3);
  - `eyebrow_type` 28:0 w5 (was 3);
  - `neck_line_type` 28:5 w3 (was 2);
  - `nose_type` 30:0 w6 (was 3);
  - `upper_lip_type` 31:0 w3;
  - `lower_lip_type` 31:3 w3.

  On PES 15 the nose field holds 3 × `nose_type` plus the code of `pes15_nose_angle`, a
  three-value slider: -1, 0 and 1 are coded 1, 0 and 2. On 16 to 21 it holds `nose_type` alone,
  and `pes15_nose_angle` has no field.
- **Colours and two-bit lists.** These exist on every version. Each colour channel is 6 bits,
  0 to 63:
  - `eyebrow_color_red` 35:0, `eyebrow_color_green` 36:0, `eyebrow_color_blue` 37:0;
  - `eyebrow_style` 35:6 w2 (0 normal, 1 thick, 2 thin);
  - `eyebrow_density` 36:6 w2 and `eyebrow_thickness` 37:6 w2 (0 to 3; their labels were not
    read);
  - `facial_hair_color_red` 39:4, `facial_hair_color_green` 40:2, `facial_hair_color_blue` 41:0;
  - `facial_hair_thickness` 41:6 w2.

  "Same as Hair Colour" in the editor copies the colour and stores no flag.
- **`pes15_eye_white` 38:6 w2.** 0 to 3: higher values make the eye whites brighter. It is
  stored on every version but drawn by PES 15 only, and PES 15's editor has no control for it.
- **`[appearance.hair]`, every version:**
  - `color_red` 31:6, `color_green` 32:4, `color_blue` 33:2, 6 bits each;
  - `accessory` 39:0 w1 (on PES 15 the game sets it with the variation);
  - `accessory_color` 39:1 w3 (the eight colours of the gloves row);
  - `color_preset` 43:4 w4.

  `color_preset` is 0 for the custom colour (the RGB above), or the editor's preset 1 to 8,
  stored through the game's internal ids: 1→5, 2→6, 3→7, 4→1, 5→3, 6→2, 7→8, 8→4. Choosing a
  preset keeps the stored RGB.
- **`[appearance.hair]`, PES 16 to 21:**
  - `style` 34:0 w4;
  - `cropped_style` 34:4 w4;
  - `length` 38:0 w3;
  - `wave_level` 38:3 w3 (stored plus 1);
  - `variation` 42:4 w8;
  - `front_style` 44:0 w3;
  - `parted` 44:3 w3;
  - `side_back_style` 44:6 w3;
  - `hairline` 45:1 w3;
  - `forehead_width` 45:4 w4.

  Labels and stored values were measured on PES 21. PES 16 to 20 are assumed equal: every cup
  save stays inside them.
- **`[appearance.hair]`, PES 15:**
  - `pes15_variation` 34:0 w8, stored as the editor's grid position minus 1;
  - `pes15_length` 38:0 w3;
  - `pes15_type` 38:3 w3.

  PES 15's "Default" hairstyle is the edited-hair flag clear, which is a compiler-owned
  `PlayerField`.
- **Unchanged:** `gloves` 0:0, `gloves_color` 0:1 w3, `skin_color` 23:0 w3, `iris_color` 42:0
  w4.
- **Measured unused**, with the value each is written as:
  - every version: 3:4, 4:0 and 19:4 (4 bits each, 7), and 30:6 w2 (1);
  - PES 16 to 21: bytes 46 to 49 (0);
  - PES 15: 42:4 w8 (0x77), byte 44 (0) and byte 45 (0x37).

  The written values are what the games' own editors and facial reset write. The games' tables
  spread the first three over 0 to 14 and 30:6 over 0 to 2, yet no render on any version moved
  with them. PES 15 shows a change of 1 in 255 over the skin for 30:6, which is invisible.

**What lands.** One implementation step, briefed to the sidekick:

- `IngameFaceField` gains a row for each field above, in the one table in
  `schema/ingame_face.rs`. Each row also names the versions it exists on: all, PES 15, or PES 16
  to 21.
- The measured-unused bits become a second table in the same file, of bit runs with their
  written values and versions.
- The completeness test's third carrier becomes that table, so each version's run is covered
  bit for bit by exactly one field row or one unused row.
- `IngameFace::undecoded` and `write_undecoded` go with `[appearance.unknown]`, which leaves
  `settings.toml` and Team TOML. Every field is a key, with the key table, ranges and defaults in
  `aesthetics_export/settings_toml.md`.
- Applying a `settings.toml` writes the whole run: every key, absent ones at their default, and
  every unused bit at its written value. So a compiled run holds nothing of the save, as for
  every other key.
- A Team TOML import keeps its "absent = untouched" rule, so the unused bits keep the save's
  values.
- Cross-version conversion converts the run field by field (`operations.md` "Cross-version
  player conversion"), and PES 15's face caps are corrected there.

## Unmodeled payload sections

The crate reads five sections of the payload (the player, appearance, team, roster and tactics
records of `spec/pes_savefile/container.md` "Payload layout per version") and their counts,
and writes every other byte back as read. A census of every cup save on the maintainer's
machine (PES 16: 28 distinct saves, 17: 73, 18: 9, 19: 23, 21: 15; 15 and 20 the fixture alone)
measured what those other bytes hold, so that modeling any of them starts here and not from a
new reading of the payload. The schema tables cover 12 to 27% of a payload's bits (PES 15
17.9%, 16 17.2%, 17 19.7%, 18 26.7%, 19 15.7%, 20 12.9%, 21 12.4%), and almost all of the rest
is empty capacity: each section is a fixed array of slots (25,000 player slots on 15/16,
21,000 on 17/18, 32,000 on 19, 30,000 on 20/21; 650 to 850 team slots; 715 to 990 roster and
tactics slots), and the
unused slots are zeros or the `0x0003FFFF` "no team" filler, identical in every save. The data
that varies between saves and no field covers is small, and every piece of it is named by the
community's edit-file wiki pages or by its content. Offsets, strides, counts and what varies
are measured; a name is the wiki's unless "measured" says an Edit-mode save moved it.

### Sections past the records

In payload order on every version, after the header, the count block, the player slots and
the team slots:

| Section | 15 | 16 | 17 | 18 | 19 | 20 | 21 | Varies across saves |
|---|---|---|---|---|---|---|---|---|
| Manager entries, 88 B (u32 id, u16 nationality, u16 picture, a byte, 79-B name) | in the team record | in the team record | 0x410118 × 850 | 0x40A35C × 850 | 0x5FECBC × 1200 | 0x94DDDC × 1500 | 0x958DA4 × 1300 | 17/18: every save; 19: 2 of 22; 21: 10 of 14 |
| Competitions | 0x4A947C, 92 B × 100 | 0x4C1B1C, 92 × 100 | 0x422548, 124 × 50 | 0x41C78C, 124 × 50 | 0x61893C, 760 × 65 | 0x96E17C (49,528 B, not 65 × 760: the stride is undetermined) | 0x974C84, 760 × 65 | 16: 21 saves; 17: 65; 18 holds the cup's competition names; 19/21: little |
| Stadium names (u32 id + name) | 0x4AB86C, 128 × 100 | 0x4C3F0C, 128 × 100 | 0x423D80, 128 × 40 | 0x41DFC4, 128 × 45 | 0x624A34, 128 × 65 | 0x97A2F4, 128 × 64 | 0x980D7C, 188 × 65 | 16: 21 saves; 17: 64; 18/19: identical |
| Kit (strip) entries, keyed by the u32 at +0 ("Kit slots and the strip table") | 0x4AEA6C, 88 × 2500 | 0x4C710C, 88 × 2500 | 0x425180, 132 × 2500 | 0x41F644, 132 × 2500 | 0x626AB4, 132 × 2500 | 0x97C2F4, 132 × 2500 | 0x983D38, 132 × 2500 | 16: 25 saves; 17: every save (155 entries, 33 teams); 18: empty; 19: 17 saves; 21: every save, all 2,500 populated |
| Competition-entry team-id lists (u32 each), after the unused roster slots | 0x506654 | 0x51EC74 | 0x48FAF8, 2,888 B | 0x487FB4, 3,008 B | 0x69DF4C, 3,392 B | 0xA00C0C, 4,656 B | 0xA08650, 4,656 B | 17: 65 saves; 18: 2; 19: 3; 21: 3 |
| Unused tactics slots to the payload's end | 397 KB | 372 KB | 146 KB | 130 KB | 56 KB | 333 KB | 333 KB | identical |

The 17 to 21 starts are the wiki's, each confirmed by the census (the section's stride starts
where the page says); the 15, 16 and 20 starts were located by content (the 92-byte and
128-byte strides, the manager and stadium names). The header's count block (0x3A to 0x4C on
15/16, 0x62 to 0x7C on 17 to 21) holds each table's u16 count: competitions, stadiums,
managers, team-player tables, game plans, and the kit entries (0x68 on 17, 0x6C on 18 to 21),
measured equal to the table's non-empty entries. On 17 to 21 the header's bytes 0x30 to 0x4F
differ in every save: the game rewrites them on every Edit-mode save (a digest or random,
inferred), and a save patched with them left as they were loads and plays, so leaving them as
read is safe. A PES 18 save also rewrote 0x0C to 0x0D and 0x54 to 0x55 (`06 29` to `76 27`,
both pairs), unexplained; PES 16 rewrites no header byte.

### Kit slots and the strip table

Every version's team record holds a block of ten u32 kit slots, each `(team id << 14) |
strip number`: the outfield 1st to 4th strips are 0 to 3 (the editor's Add menu offers those
and the goalkeeper's 1st; 4 to 8 occur on one team of one save), the goalkeeper's 1st is 0x80,
and an unused slot is 0xFFFFC000. The block is at +0x1C on 17 and 18 (measured on 18), +0x20
on 19 (measured) and +0x30 on 20/21 (measured on 21); on 15/16 bytes 24 to 63 are set and
uncovered, the same size, inferred to be the same block. The spec's tables carry the
`KitSlotNumber`/`KitSlotTeam` rows for 17 alone (`spec/pes_savefile/model.md` "The team
side"), so the rows for the other versions are what modeling this adds. Measured on 18, 19 and
21: Edit > Teams > Strip > Add writes the new strip's key into the block's first unused slot
(so a block's order is the order of adding, not numeric: one team holds 0, 1, 0x80, 2, 3), adds
one entry with the same key to the strip table, which stays sorted by key (the entries after
it move down one slot), and adds 1 to the header's kit-entry count; nothing else changes.
The entry's body is the strip's design: on 20/21 a `FF 07 00 50` word and RGB-looking
triplets (`36 36 36` in a default entry), inferred to be the kit colours; the layout is
unmapped. The maintainer's PES 21 install refuses to edit a strip ("You cannot edit this
strip"), on a cup team's 1st and on one just added, so mapping the body by editing needs
another install or another route. A team whose slots name strips with no table entry (PES
19: 158 teams own strips 0, 1 and 0x80 against 15 entries) draws them from the kit configs
the CPK carries, inferred, not tested. In the live saves: PES 18, 89 teams with slots 0, 1 and
0x80 (11 with a 4th, one with all ten) and 96 with none, the table empty; PES 19, 158 teams
with 0, 1 and 0x80 and 52 with none; PES 21, every table entry populated, default teams
included.

### The team record

The tables cover 15 to 26% of the team record's bits. The uncovered bytes by version, with the
wiki's names (what a Save editor's team page would model):

- **15/16** (456 B): 4 to 17 emblem, home stadium, nationality, callname, coach nationality and
  image, turf, net, seats and sideline; 18 to 19 edit flags (the tables' `EditedName` at 0x12:0
  is the wiki's "Unknown A", and its "names edited" is 0x12:7); 24 to 63 the kit slots; 64 to
  75 three rival ids; 76 to 147 0xFF (145 to 147 set in about 600 records); 222 to 342 the
  custom stadium name; 343 to 406 four banner texts at a 16-byte stride; 407 to 455 the
  manager's name.
- **17/18** (480 B): 4 to 17 manager id, emblem, stadium, nationality, callname; 19 to 27
  stadium settings and edit flags (bits 215 and 216 set in about 10,400 of 15,221 records); 28
  to 67 the kit slots (uncovered on 18 alone); 68 to 79 rivals; 80 to 151 0xFF with
  banner-edited flags at 149 to 151; 226 to 346 the custom stadium name; 347 to 410 banners.
- **19** (416 B): 8 to 22; 25 to 31 the wiki's "Unknown B to E" (by 21's layout the goal-net
  colours, media backdrop, stadium settings, the 10-bit edited-settings mask and partner
  images; the census ran them to 51 and started the kit slots at 52, but the measured block
  starts at 32 and the rivals follow it at 72 as on every other Fox version, so the byte
  counts between 25 and 51 are not to be trusted); 32 to 71 the kit slots; 72 to 83 rivals; 85 to 87
  banner flags; 162 to 178 the stadium name; 283 to 344 banners.
- **20/21** (528/588 B): 8 to 22; 25 to 33 goal-net colours, media backdrop RGB, edited-stadium
  flags, goal, turf, sideline and seats, the 10-bit mask; 36 to 47 three partner-image ids
  (0xFFFFFFFF); 48 to 87 the kit slots; 88 to 99 rivals; 101 to 103 banner flags; 359 to 422
  banners; the 181-byte stadium name was set in no save.

Measured on PES 21: Rival Clubs slot 1 is the u32 at +0x58 (0x0003FFFF is Off), and setting it
also set bit 264, inferred to be the 10-bit mask's edited-rivals flag; the team colours are at
the tables' bits (`spec/pes_savefile/codec.md` "Reference readings the derivation flagged").

### The tactics record

The tables cover 4,120 of the record's 5,024 bits on 17 to 21 (4,088 of 4,128 on 15, 4,112
of 4,160 on 16, where no uncovered bit is set in any save). On 17 to 21 the uncovered bits
that real saves set are these. Bytes 516 to 523 on 19 to 21 are lineup slots 32 to 39
of the wiki's 40-byte lineup at 0x1E4 (the tables cover slots 0 to 31 as the starting eleven
and the bench); on PES 20, 218 of 220 records hold exactly 32 to 39 there. Byte 537 bit 0 is
Switch Preset Tactics (measured on 21: On to Off cleared it; set in 1,529 of 8,388 PES 19
records and 1,185 of 3,180 PES 21 ones) and byte 539 is Auto Lineup Select (measured on 21:
"Unedited" is 0 and the next entry right, "By Ability", is 2, on which the game re-picked the
lineup at once and swapped two lineup bytes; the other entries were not stepped through). On
17/18 that setting is byte 530 bit 1 (values 2 and 3 in 288 and 25 records). The wiki's PES 19
page lists its offsets after the lineup 3 bytes short; the tables and the data agree with each
other.

### The player record

- Every version: bytes 4 to 7 are the commentary-name player id (a 0x1xxxx value in most
  records, 0xFFFFFFFF in a few; a PES 16 Base Copy rewrites it, with the appearance block's
  base-copy id, to the copied player's); bytes 8 to 9 are 0xFFFF on 15 to 18.
- **PES 15** bits 149 to 155 (between goalkeeping and the free-kick motion) hold 99 in every
  record (on 16 those bits are dribbling); bits 344 to 345 are set in 13 of 20 records.
- **PES 16** bits 222 (0x1B:6, the wiki's "Unknown B") and 362 (0x2D:2, "Unknown D") are set
  together on the 23 players of one team (/gd/, 720) in 19 of the 28 saves and nowhere else.
  Measured: no Edit setting moves them (basic settings, motion, ability with injury resistance
  at its top, skills, COM styles and base copy were tried; positions, playing style,
  appearance, name and create player were not), nothing in the editor shows them, and the game
  keeps them through load, edit and save. Retained, meaning unknown, not editable.
- **PES 17** three one-record bits in byte 51 (noise).
- **PES 18** bits 252 to 254 (0x1F:4, 3 bits; on 17 the dribbling-arm motion sits there). No
  Edit setting moves them, and a dribbling-arm change moved the tables' own field at bit 295,
  so the tables' PES 18 motion offsets hold and these bits are not that motion. Konami's own
  players hold 1 to 7 on all 13,800 of the default save (1: 6,263, 2: 3,730, 3: 2,029, 4:
  1,057, 5: 444, 6: 195, 7: 82; the mean ability rises from 60.6 at 1 to 69.6 at 7, with no tie
  to the arm motions), the live cup save's rostered players 0, and one cup save's Invitational
  teams 2,827 nonzero. Kept through load, edit and save. Retained, meaning unknown, not
  editable.
- **PES 20/21** bytes 176 to 239 are the wiki's 64-byte "Print Name (National Team)" (varying
  at 176 to 191 on 21).
- The appearance block has no uncovered bit; the 20/21 roster record has 40 bytes set in no
  save.

An Edit-mode save also sets the four edited flags at +0x78 (bits 960 to 963: face, hair,
physique, strip) on a Fox player whose screens were visited, whether a value changed or not;
a player whose only visit was the hairstyle picker, confirmed on the entry it showed, stayed
byte-identical, so which screen sets them is not isolated.

### Open points

Not determined by the census: PES 20's competition stride (49,528 B is not 65 × 760); PES 15's
header 0x4A reading 1855 over an all-empty kit table; PES 16's competition-entry start (the
ids begin 128 B before the wiki's size implies); the strip entries' layout beyond the key and
the colour-looking bytes; the 32 header bytes; the edit flags' meanings (the wiki's words,
unmeasured). Not measured in Edit mode: the manager entries, competitions, stadium names, the
commentary-name id, the print name, and the team record's emblem, stadium and banner bytes.
