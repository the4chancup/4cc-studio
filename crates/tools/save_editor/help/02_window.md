The Save editor's window opens a save (`EDIT00000000`, or a copy of it) and edits it.
**Open…** picks the file. Once a save is open the toolbar shows its path, marked
`• unsaved changes` while edits wait unsaved.

**Teams and players.** The dropdown picks a team, or ALL for every team of the save. The
list below it holds one row per player: shirt number, position, the team when the list
crosses teams, and the name shown as the game draws it, colors included. A `●` on a row
marks a player with unsaved changes. The filter box above the list finds a player of any
team by part of his name, shirt name or id, so finding him does not need his team first.
With one team picked and the filter empty, drag a row to move the player to a new place in
the list; the shirt number and the in-game face and hairstyle belong to the place, so they
stay where they were.

**Editing.** Click a row to select the player; the fields right of the list edit his
general info: name, shirt name, height, weight, age, nationality, shirt number and
captain. A `\x11c` plus 8 hex digits inside a name is a color code and is kept as typed.
Every edit is undoable: **Undo** and **Redo** step through them (Ctrl+Z, Ctrl+Y or
Ctrl+Shift+Z), and a drag or a run of typing on one field is one undo step.

**Saving.** **Save** or Ctrl+S writes the edits to the file the save was opened from (a
file it replaces is kept once as a `.bak` copy). Saving is refused while the save's own
PES version is running: the game writes its save back when it closes, over the one just
written, so close the game first. A PES of another version running does not stop it.
