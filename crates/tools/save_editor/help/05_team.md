Clicking a team's row in the list opens the team view: the team's name,
three-letter short name and two colours on the first lines, then the manager and
stadium IDs with the **Kit slots…** button, and the Tactics below. **Kit
slots…** opens the kit slot assignments in a dialog (PES 2017 only) and closes
on OK. A field the save's version does not hold stays in its place, disabled,
with a note naming the first version that has it.

The name and short name fields keep only what the save can hold: the name up
to the save's byte limit (an accented letter counts more than one), the short
name up to three letters, shown upper case. A colour opens the same picker as
elsewhere. The save keeps 64 steps per channel, so the colour written is the
nearest one the save can hold, and the hex beside the field shows what the
save actually stores. A bound kit slot is shown as the team id it points at,
and the extra data the game keeps alongside is left as it was.

**The Team menu** works on the selected player's team and writes every filled
roster slot in one undoable step:

- **Set edited flag** marks every player of the team as edited.
- **Set boots and gloves IDs for everyone…** opens a small dialog: the boots
  can stay unchanged, count up from a start ID (slot 1 gets the start, slot
  2 the next, and so on) or take one ID for all, and the gloves can stay
  unchanged, take the same IDs as the boots or take one ID for all.
- **Set player names to positions** renames every player of the team to his
  registered position's full name.
- **Clear visual flags** (Ctrl+F) clears the face, hair, physique and strip
  edit flags on every player of the team.
- **Set FPC invisibility** writes the hide preset on every player of the
  team: long sleeves, a tucked shirt and short socks, and a boots or gloves
  ID of 0 takes the hide ID from Settings (55 and 11 unless changed, while a
  model's own ID is kept, as in the Appearance section). **Remove FPC
  invisibility** writes the show preset on them all. On PES 2015 to 2017
  both also turn a Custom skin into Light.

The boots and gloves item, Clear visual flags and the FPC items stay
disabled while the aesthetics lock is on (see the Appearance section), because
they write what the team's export owns and a hand edit would be overwritten
the next time the team is recompiled or patched. Ctrl+F does nothing while
the lock is on.
