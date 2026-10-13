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
