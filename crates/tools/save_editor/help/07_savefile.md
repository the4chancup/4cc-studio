The **Savefile** menu works on the whole save:

- **Output rosters to TSV…** writes the roster sheet, the file members
  paste into a spreadsheet to check a cup's teams: every team of the
  save, hidden ones included, its name on a line of its own, then one
  tab-separated row per player (shirt number, name, position, Finishing,
  a `(C)` on the captain, height, playing style, skills and COM styles).
  A save dialog asks where to write it, `EDIT.tsv` the default name. It
  changes nothing of the save.
- **Fix database…** is a maintenance repair behind a confirmation: it
  repairs base copies on PES 2015 to 2018 (a stray copy id clears, a
  flag that points at nothing clears; later games' records have no
  base-copy flag, so their copy ids are kept), unmarks the face, hair,
  physique and strip edits on every player and the strip flag on every
  team, and resets the kit slots to their defaults on PES 2017. The
  whole repair is one undoable step. It stays disabled while the
  aesthetics lock is on (the Appearance tab's **Edit aesthetics anyway**
  switch), since it clears what the teams' exports own.
