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
stripped save (below) it warns that the game takes the look from the database
tables while the motions are still read.

A player marked **stripped** takes his look from the database tables, not from
his save record (his appearance id is -1): the game's Edit mode restores the id
on any change, so looks for him go through the export, never this tab. His
motions, the Motion flag and, on PES 2018, the Base copy flag still apply:
they sit in the part of the record the game reads even for a stripped player. When a save of PES 2018 or
newer opens with some players stripped and some not, a window lists the ones
still carrying an appearance id, and **Strip again** sets them back to -1 in
one undoable step.

**Randomize** draws new values for the hunching, arm movement, kick and
celebration motions, each within the range that game's Edit mode offers; the
dribbling motion is left as it is. One Undo restores them all.

The **Full Player Customization** group, last on the tab, is where FPC
invisibility lives: the save's own reading of the player, then **Hide body**
and **Show body**. The reading says "FPC: hidden body" when the record's
boots ID is the convention's nonexistent one (55) or his skin is Custom,
"FPC: body shown" otherwise. A player hidden by his model's own boots ID,
such as a compile-allocated one, reads "body shown": the save cannot tell
his model hides him, which is why these are two buttons and not a checkbox.
Hide body writes long sleeves, a tucked shirt, short socks, boots 55 and
gloves 11 where the IDs are 0 (a model's own boots or gloves ID is kept);
Show body writes short sleeves, untucked, standard socks, and boots and
gloves 0 where they are 55 and 11. Under the reading the group lists the
strip settings that would interfere with a hidden body (inners or
undershorts that break the hiding, taping that still shows, the gloves
checkbox with ID 0 giving winter gloves): they are reported, never
changed for you. The list follows the save's own reading, so it only
covers a player the save calls hidden: one hidden by his model's own
boots ID (a compile-allocated one such as 627) reads "body shown" and
his interfering settings are not listed, though the game hides his body
all the same. The Team compiler's check knows the folder's FPC marker,
so it reports them for a compiled team.
