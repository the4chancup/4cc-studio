The Save editor's window opens a save (`EDIT00000000`, or a copy of it) and edits it.
**Open…** picks the file. Once a save is open the window's title bar shows its path, in
square brackets after the editor's name, and the toolbar shows `• unsaved changes` while
edits wait unsaved.

**Teams and players.** The dropdown picks a team, or ALL for every team of the save;
picking a team selects its row, so the right side shows the team's fields and tactics
(ALL keeps what you had selected). The
list below it holds a row per team and, under it, one per player: the team's row shows
its id, its two colours and its name followed by `[click for tactics]`, a player's row
his shirt number, position, the team
when the list crosses teams, and the name as the game draws it, colors included. A `●` on
a row marks a player or the team itself with unsaved changes. The filter box above the
list finds a team, or a player of any team, by part of a name, shirt name or id, so you
can find someone without knowing their team first. With the filter empty, drag a player
row onto another row of his team to move him to that place; a drop on another team's row
does nothing. The shirt number and the in-game face and
hairstyle belong to the place, so they stay where they were.

**Editing.** Click a player's row to select him: the right side shows his id and
name, then three columns of his fields, and the collapsed **Appearance** section
underneath. His Name and Shirt labels count what the field holds (`(12/61)` reads
as 12 used of 61) and turn amber when it is longer than every version can hold. Click a
team's row to select the team: the right side shows the team's fields and its tactics.
A `\x11c` plus 8 hex digits inside a name is a color code, and the field keeps it as typed.
Every edit is undoable, and **Undo** and **Redo** step through them (Ctrl+Z, Ctrl+Y or
Ctrl+Shift+Z); a drag or a run of typing on one field is one undo step.

**Quick actions.** Under the list, one **Make \<tier\>** button per tier of the AATF
rules in use gives the player that tier's values (stats, form, injury resistance, weak
foot, name color) as one undoable edit. **Set stats** sets every ability stat to the
number beside it (40 to 99). They act on the selected player, so a team row leaves them
greyed out. When the rules file does not load, its error shows in place of the Make
buttons.

**AATF violations.** **Panels** in the toolbar opens a floating window listing a
team's violations against the AATF rules
in use, grouped per player. **Current team** checks the team of the selected row,
a player's own or a team row's; with nothing selected it checks the team the
selector names (with ALL picked and nothing selected, no team is open).
**Select teams** lists every team of the save to
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
selects that player; a `FieldedNotA` row, about a position he is fielded at,
selects his team's row instead, on the named preset's first formation that
carries the violation's `✖`. On the tactics pitch that cross stands just left
of his marker, on every formation the preset fields him at that position. The
toggle is remembered, so it is still on the next time the editor is opened.

**Saving.** **Save** or Ctrl+S writes the edits to the file the save was opened from (a
file it replaces is kept once as a `.bak` copy). While the save's own PES version is
running, saving first asks you to close the game's Edit mode if it is open. The game
writes its own copy of the save when you leave Edit mode, so a save written while it is
open would be lost. After saving, use the Load option in Edit mode to see the changes in
game. A PES of another version asks nothing.

**Unsaved changes.** Closing the window or opening another save with unsaved edits asks
first, over a table of each changed field with its value in the file and in the editor.
Save writes the edits, Discard loses them, and Cancel keeps everything as it was.
