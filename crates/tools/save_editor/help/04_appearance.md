The **Appearance** tab holds the player's look: the game's own edit flags, his
physique, his strip style, his motions, and his skin and iris colours.

**It is locked on purpose.** A player's look belongs to his team's export: the
export's `settings.toml` and models decide it, and the Team compiler writes it
into the save (through the aesthetics patch on older games, through the database
tables on PES 2018 and newer). A hand edit here would be silently overwritten the
next time the team is recompiled or patched, so the tab opens read-only at every
start of the tool.

**Edit aesthetics anyway** unlocks the tab for the rest of the session, for the
cases the rule does not cover: a team with no export, or a cup-night fix when
recompiling is not an option. Opening another save keeps it unlocked; starting
the tool again turns it back off. On a save of PES 2017 or older the note under
the switch warns that a later patch overwrites whatever you edit by hand; on a
stripped save (below) it warns that the game does not read these fields at all.

A player marked **stripped** takes his look from the database tables, not from
his save record (his appearance id is -1): the game's Edit mode restores the id
on any change, so looks for him go through the export, never this tab. When a
save of PES 2018 or newer opens with some players stripped and some not, a
window lists the ones still carrying an appearance id, and **Strip again** sets
them back to -1 in one undoable step.

**Randomize** draws new values for the hunching, arm movement, kick and
celebration motions, each within the range that game's Edit mode offers; the
dribbling motion is left as it is. One Undo restores them all.
