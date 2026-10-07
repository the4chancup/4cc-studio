# Number atlases: where each game keeps each digit

The measurement behind `NUMBER_ATLAS_SLOTS` (`team_compiler` `processing/kit_layout.rs`, step
4.32): how a `_back`, `_chest` or `_leg` atlas in one engine's arrangement is re-arranged into
the other's. Made 2026-10-07 from the stock atlases of both games, extracted to
`E:\PES2017\Data\dt34_win_files\common\character0\model\character\uniform\texture` (797 of each
suffix) and `E:\PES2021\Data\dt34_g4_files\Asset\model\character\uniform\texture\#windx11`
(1,357 of each); 465 kit stems exist in both. FTEX files are read through the 19-to-16
converter's `Ftex` module (`C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib`), as
`kit_leg_atlas.py` does.

## What was found

- **PES 17** stacks the ten digits in a column, 0 at the top, each upright inside an equal
  tenth of the length (`union17.txt`: every stock glyph's ink lies in [204d + 6, 204d + 204]
  of a 2048-long column, 8 px apart; across, within 8 to 120 of 128).
- **PES 21** lays them in a row, 0 on the left, but **not in equal tenths**: each glyph sits in
  a fixed slot about 160 x 256, the slots 40 to 50 px apart and about 150 px empty after the 9
  (`union21.txt`: the ink union of all 1,352 clean atlases per digit, 0-158, 200-300, 346-494,
  546-694, ..., 1746-1896). A 160 x 256 slot has the proportions of a PES 17 cell
  (128 x 204.8), scaled by 1.25.
- The same team's glyph in both games keeps its proportions (`glyph_pairs.txt`: the median
  change of aspect is 1.00, where stretching a cell onto a tenth would give 1.28), and its
  scale is 1.234 to 1.263 (`slot_fit.txt`, pairs drawn in the same proportions only).
- `slot_fit.txt` fits each scaled cell's left edge: 0, 191, 344, 541, 740, 940, 1141, 1339,
  1540, 1740 (medians, px of a 2048-long row). The rounded rule 0, 190, then 200d - 60
  reproduces the stock rows as well (`remap_test.txt`, IoU of ink against the same team's
  stock PES 21 atlas, 465 stems per suffix: the rule's median 0.676 for `_back`, the fitted
  edges' 0.673; stretching each cell onto a tenth 0.155; keeping the pixel scale centred in
  a tenth 0.047; the median is below 1 because many teams changed font between the games,
  and the example `a0150p1` reaches 0.871). `a0150p1_back_compare.png` stacks, from the top,
  the stock PES 21 row, the PES 17 column re-arranged by the rule, by stretching, and by
  keeping the scale.
- An atlas outside its glyphs holds the glyph's color at alpha 0 in both games
  (`gap_pixels.py`), so filtering at a glyph's edge does not darken it.
- `_name` atlases put their letters in the same places in both games (the research's
  `name_census.py`, within 0.002 of the width), so they are left alone.
- Neither game has number meshes: both compose a player's number texture at run time
  (PES 21's shaders sample one composed texture; PES 17's executable names
  `NumberTextureCreateRequest`). Where each engine samples a digit is in code, not in data,
  and was not found there; the table reproduces Konami's own layout instead.

## The table

In units of a 2048-long atlas, the long axis first:

| digit | column cell (PES 15-17) | row slot (PES 18-21) |
|---|---|---|
| d | [204.8 d, 204.8 (d + 1)) x the whole width | [start_d, end_d) x the whole height |

start = 0, 190, 340, 540, 740, 940, 1140, 1340, 1540, 1740; end_d = min(start_d + 160,
start_{d+1}), so every slot is 160 long except digit 1's, [190, 340), which ends where the 2's
begins (the stock 1's ink ends at 300, the 2's starts at 346). A cell maps onto its slot by a
uniform scale of 1.25 (a slot onto its cell by 0.8), top edges together; the part of a scaled
cell past its slot's end is left out. A row is the column's length long and an eighth of it
high; a column a sixteenth of it wide.

## Reproducing

Run from this folder, in order (the CSVs they write are not committed):

```
python glyph_census.py 17 && python glyph_census.py 21   # glyph_census_<game>.csv
python glyph_union.py 17 > union17.txt && python glyph_union.py 21 > union21.txt
python glyph_pairs.py > glyph_pairs.txt                     # glyph_pairs.csv
python slot_fit.py > slot_fit.txt
python remap_test.py > remap_test.txt                       # also png/ examples
python gap_pixels.py
```

`glyph_cells.py` holds the loaders and the ink rule (alpha above 32 where the texture has
alpha, else luma); `glyph_runs.py` the runs of ink along an axis.
