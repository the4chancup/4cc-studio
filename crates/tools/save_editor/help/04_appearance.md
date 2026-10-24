The **Appearance** tab holds the player's look: the game's own edit flags, his
physique, his strip style, his motions, and his skin and iris colours.

The **Cutscenes** group comes first and stays editable while the rest is locked:
star and playing attitude only change how the player appears in cutscenes, so
they are not part of the look the team's export owns.

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
boots ID is the hide ID from Settings (55 unless changed) or his skin is
Custom, "FPC: body shown" otherwise. A player hidden by his model's own
boots ID, such as a compile-allocated one, reads "body shown": the save
cannot tell his model hides him, which is why these are two buttons and not
a checkbox. Hide body writes long sleeves, a tucked shirt, short socks, and
the hide IDs from Settings (55 and 11 unless changed) where the IDs are 0 (a
model's own boots or gloves ID is kept); Show body writes short sleeves,
untucked, standard socks, and boots and gloves 0 where they are the hide
IDs. On PES 2015 to 2017 both also turn a Custom skin into Light. Changing
the setting rewrites no save: a player hidden with the old ID then reads
"body shown", and Show body leaves his ID alone (it is no longer the hide
ID). Under the reading the group lists the strip settings that would
interfere with a hidden body (inners or undershorts that break the hiding,
taping that still shows) and the gloves checkbox with ID 0, which gives
winter gloves to any player: they are reported, never changed for you. The
inners, undershorts and taping follow the save's own reading, so they are
listed only for a player the save calls hidden: one hidden by his model's
own boots ID (a compile-allocated one such as 627) reads "body shown" and
they are not listed for him, though the game hides his body all the same.
The Team compiler's check knows the folder's FPC marker, so it reports them
for a compiled team.
