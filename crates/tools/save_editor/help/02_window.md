The Save editor's window opens a save (`EDIT00000000`, or a copy of it) and edits it.
**Open…** picks the file. Once a save is open the toolbar shows its path, marked
`• unsaved changes` while edits wait unsaved.

**Teams and players.** The dropdown picks a team, or ALL for every team of the save. The
list below it holds one row per player: shirt number, position, the team when the list
crosses teams, and the name shown as the game draws it, colors included. A `●` on a row
marks a player with unsaved changes. The filter box above the list finds a player of any
team by part of his name, shirt name or id, so you can find him without picking his team
first. With one team picked and the filter empty, drag a row to move the player to a new
place in the list. The shirt number and the in-game face and hairstyle belong to the
place, so they stay where they were.

**Editing.** Click a row to select the player. The bar over the tabs edits his
team's name, short name and two colours on any tab. The tabs group as
**Player** (Abilities & Skills, Appearance) and **Team** (Tactics, Team
settings). The two player tabs open on his name and general info: shirt name,
height, weight, age, nationality, shirt number and captain. A `\x11c` plus 8
hex digits inside a name is a color code, and the field keeps it as typed.
Every edit is undoable, and **Undo** and **Redo** step through them (Ctrl+Z, Ctrl+Y or
Ctrl+Shift+Z); a drag or a run of typing on one field is one undo step.

**Quick actions.** At the top of the Abilities & Skills tab, one **Make \<tier\>**
button per tier of the AATF rules in use gives the player that tier's values
(stats, form, injury resistance, weak foot, name color) as one undoable edit.
**Set stats** sets every ability stat to the number beside it (40 to 99). When
the rules file does not load, its error shows in place of the Make buttons.

**AATF violations.** **AATF…** at the bottom of the quick actions, or **Panels** in the
toolbar, opens a floating window listing a team's violations against the AATF rules
in use, grouped per player. **Current team** checks the team the selector names (with
ALL, the selected player's team). **Select teams** lists every team of the save to
check several at once, which takes a moment while the results come in. Clicking a
player's violation selects him. **Show suggestions** adds, below the violations, what
each player could still use within the rules. **Validate rules** checks the rules
file itself and lists everything it finds wrong with it. **Save a copy** writes the
built-in official ruleset out to a file.

**Comparator.** **Panels** in the toolbar, **Comparator…**, opens a floating
window comparing the open save, edits included, against a second save:
**Pick…** chooses it and refuses one of another version, naming both. The
**Show** row narrows the list to All, Gameplay or Aesthetics, and the team
dropdown to one team. Rows group by team and by player: each row shows a
field's name and its value in the picked save and in the open save, and a mark
sits beside a player stripped in one save or both. Clicking a row selects
that player in the editor. A comparison takes a moment on a big save, and the
window says so until the rows land. **Export…** writes the picked scope's
differences over the whole save as a text file.

**Live AATF.** The toolbar's **Live AATF** toggle checks the open team after every
edit, in a panel docked at the bottom of the window: the violations left, grouped
per player as the floating window lists them, under `N issues · <ruleset>`, or
`<team> passes <ruleset> with no issues` once the team passes. Clicking a row
selects that player. The toggle is remembered, so it is still on the next time
the editor is opened.

**Saving.** **Save** or Ctrl+S writes the edits to the file the save was opened from (a
file it replaces is kept once as a `.bak` copy). While the save's own PES version is
running, saving first asks you to close the game's Edit mode if it is open. The game
writes its own copy of the save when you leave Edit mode, so a save written while it is
open would be lost. After saving, use the Load option in Edit mode to see the changes in
game. A PES of another version asks nothing.

**Unsaved changes.** Closing the window or opening another save with unsaved edits asks
first, over a table of each changed field with its value in the file and in the editor.
Save writes the edits, Discard loses them, and Cancel keeps everything as it was.
