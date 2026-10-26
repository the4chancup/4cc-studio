The **Savefile** menu works on the whole save:

- **Output rosters to TSV…** writes the roster sheet, the file members
  paste into a spreadsheet to check a cup's teams: every team of the
  save, hidden ones included, its name on a line of its own, then one
  tab-separated row per player (shirt number, name, position, Finishing,
  a `(C)` on the captain, height, playing style, skills and COM styles).
  A save dialog asks where to write it, with `EDIT.tsv` as the default
  name. It changes nothing in the save.
- **Fix database…** is a maintenance repair behind a confirmation: it
  repairs base copies on PES 2015 to 2018 (a stray copy id is cleared, and
  a copy flag that points at nothing is cleared; later games' records have
  no base-copy flag, so their copy ids are kept), clears the face, hair,
  physique and strip edit flags on every player, and on PES 2017 clears
  every team's strip flag and resets the kit slots to their defaults. The
  whole repair is one undoable step. It stays disabled while the
  aesthetics lock is on (see the Appearance section), since it clears what
  the teams' exports own.
