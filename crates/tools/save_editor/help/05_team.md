The team bar above the tabs edits the selected player's team's name,
three-letter short name and two colours; the **Team settings** tab holds the
manager and stadium IDs and the kit slot assignments (PES 2017 only). A field
the save's version does not hold stays in its place, disabled, with a note
naming the first version that has it.

The name and short name fields keep only what the save can hold: the name up
to the save's byte limit (an accented letter counts more than one), the short
name three letters, shown upper case. A colour opens the same picker as
elsewhere; the save keeps 64 steps per channel, so the colour written is the
nearest the save holds, and the hex beside the field reads what the save
actually stores. A bound kit slot is shown as the team id it points at; the
bits the game keeps alongside are left as they were.

**The Team menu** works on the selected player's team and writes every filled
roster slot in one undoable step:

- **Set edited flag** marks every player of the team as edited.
- **Set boots and gloves IDs for everyone…** opens a small dialog: boots
  unchanged, counted up from a start ID (slot 1 gets the start, slot 2 the
  next, and so on), or one ID for all; gloves unchanged, the same as the boot
  IDs, or one ID for all.
- **Set player names to positions** renames every player of the team to his
  registered position's full name.
- **Clear visual flags** (Ctrl+F) unmarks the face, hair, physique and strip
  edits on every player of the team.
- **Set FPC invisibility** writes the hide preset on every player of the
  team: long sleeves, tucked, short socks, and the hide IDs from Settings
  (55 and 11 unless changed) where the IDs are 0 (a model's own boots or
  gloves ID is kept, as on the Appearance tab). **Remove FPC invisibility**
  writes the show preset on them all. On PES 2015 to 2017 both also turn a
  Custom skin into Light.

The boots and gloves item, Clear visual flags and the FPC items stay
disabled while the aesthetics lock is on (see the Appearance tab): they
write what the team's export owns, so a
hand edit would be overwritten the next time the team is recompiled or
patched. Ctrl+F does nothing while the lock is on.
