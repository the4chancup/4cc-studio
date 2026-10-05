# 4cc Studio — Aesthetics export plan: settings.toml

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player settings in exports (settings.toml)

Aesthetic settings that used to live only in the savefile move into the export as TOML, and the
compile carries them to the game. **How depends on the engine:**

- **Fox (PES 18–21): in the database tables the output CPK carries.** The game takes a player's
  appearance from `PlayerAppearance.bin`, his boots from `BootsList.bin` and his gloves from
  `GloveList.bin` whenever his savefile appearance record's player id is -1 (`0xFFFFFFFF`), player
  by player; a record whose id is set wins over the tables (in-game Test 1, PES 2021: decision
  entry "aesthetics travel in the database tables"). The official save is therefore stripped once
  (every appearance id set to -1, see "Stripped save" in the [Save editor plan](../save_editor.md))
  and the compiler writes each compiled player's three rows from this file; nothing of a
  player's appearance is written to the savefile. Gloves need FoxDen's gloves patch to keep the table's value (worklog
  "Issues"); until it does, a Fox gloves row has no effect in the 4cc setup.
- **Pre-Fox (PES 15–17): in the savefile**, through the aesthetics patch the compiler writes and
  the save editor applies, as before. The database route is untested there.

**Everything in the file is optional** — including the file itself: a player folder with only
models is valid. **An absent key takes its default**, the value the template below shows for it
(0 for an unknown bit, see below), on both engines; model-derived ID assignments and FPC
directives are separate inputs. `name` and `shirt_name` are the exceptions: absent means "not
written" (see the name rules). A compiled player's appearance is therefore his export (this file,
his folder's models and markers) and nothing else: no earlier compile, savefile or installed
table contributes to it, except a boots or gloves ID whose output failed (step 1 below).

**The file can express everything the savefile's player appearance record holds.** Its schema is
`pes_savefile`'s `PlayerSettings`: all of the decoded player appearance record — physique, strip
style, wrist-tape and spectacle colours, skin and iris colour, the motion block, the eleven
ingame-face feature types, and stock boots/gloves IDs (below) — except the compiler-owned edit
flags and base-copy ID, plus the record's **undecoded bits** as raw values. The ingame-face run
carries bits no tool has decoded (hair, face-slider positions, colours beyond skin and iris; about
one cup player in twenty relies on in-game hair rather than a face model): `[appearance.unknown]`
holds them per version, a single unknown bit as a bool and a run of adjacent unknown bits as
base64, until they are measured and given names (see "Player settings model" in the
[Savefile plan](../pes_savefile/model.md), which also holds the completeness test). Completeness
matters here because this file is the **only** route by which a team's aesthetics reach the
game: the save editor's own appearance fields are read-only by default (see "Read-only
aesthetics" in the [Save editor plan](../save_editor.md)), and on Fox a stripped player's
appearance bytes are not read at all. The **template** — what
the Export upgrader and the save editor generate, and what a new player folder starts from —
therefore lists every key, one per line, each followed on the same line by the comment that gives
its range: set ones with their value, unset ones as commented lines (the boots/gloves IDs as
`""`, see below), so a team can see what is
authorable without any other document. The block below is the complete key table; `to_toml`
emits it in this order and with these comments.

**Boots/gloves IDs are authored only for stock models.** Custom boots and gloves are folder
content: local models and shared link files determine the compiler's player-exclusive or shared
assignments (see "Player folders"), and users never copy a custom model's numeric ID into this
file. A player who wears one of the game's own models has no folder to express that, so the two
top-level keys `boots_id` and `gloves_id` name a model of the **stock band**, IDs 0 to 100: the
cup's stock kit compacts Konami's boots and gloves into that range, and the compiler's per-team
blocks start at 101, so an ID above 100 is always some team's custom content and is rejected at
parse (`OutOfRange` naming the key). Besides an integer, both keys accept the empty string, which
means **default** and is the same as an absent key. Each category resolves in this order:

1. The folder's own models or a link file provide the category → the compiler-assigned ID; when
   that output fails, the player's installed ID stays (Fox: his row of the installed table;
   pre-Fox: the savefile's current ID), so no ID points at content the CPK lacks. A numeric key
   is ignored with `settings_model_id_conflict` (W).
2. A numeric key → that stock ID, whatever FPC marker the folder carries.
3. Default (`""` or absent) → the folder's FPC marker decides: `fpc_on` writes the hide preset's
   nonexistent IDs (boots 55, gloves 11); `fpc_off` or no marker writes 0 for both (the default
   boots and a pair of normal hands).

Gloves are not a goalkeeper-only model here: the cup's gloves system lets any player wear a gloves
model, which is why gloves ID 0 is a set of normal hands rather than "no gloves".

Default is what nearly every player wants: an FPC player (a face model carrying the full body,
no boots/gloves folders) needs the blank IDs, which the marker supplies without a key. Pre-Fox
local models embedded in face XML request no standalone output, so they fall through to steps 2
and 3. The template writes both keys uncommented
as `""`, the only keys shown with a value that sets nothing: every other key has a neutral value
to display, while any boots/gloves ID is a real model, and TOML has no bare `key =`. They sit at
the file's top level, next to `name`, rather than in
`[appearance]`: Team TOML embeds the `[appearance]` tables and has its own player-level
`boots_id`/`gloves_id` over the full stored range (save interchange), which two keys of the same
name inside `[appearance]` would collide with.

```toml
# settings.toml, inside a player folder. Every key is optional: an absent key takes the
# value shown here (name and shirt_name: not written). A commented key shows what can be
# set and its range. The file never references the export's models: what a player wears
# is decided by the folder contents (models present, link files pointing at shared
# folders); boots_id/gloves_id only name the game's stock models.

# true = derive from the folder name ("15 - Snuffy" gives "Snuffy"; the whole folder
# name for players.txt-mapped folders); "text" = write as is; absent = not written.
name = "Snuffy"
shirt_name = "SNUFFY"           # "text" = write as is; absent = not written
boots_id = ""                   # "" = default (fpc_on: hidden, otherwise 0); 0 to 100, a stock boots model; ignored when the folder has boots models or a boots link
gloves_id = ""                  # "" = default (fpc_on: hidden, otherwise 0); 0 to 100, a stock gloves model (0 = normal hands); ignored when the folder has gloves models or a gloves link

[appearance]
skin_color = 1                  # 0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)
iris_color = 0                  # 0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet

[appearance.physique]
neck_length = 0                 # -7 to 7
neck_size = 0                   # -7 to 7
shoulder_height = 0             # -7 to 7
shoulder_width = 0              # -7 to 7
chest = 0                       # -7 to 7
waist = 0                       # -7 to 7
arm_size = 0                    # -7 to 7
arm_length = 0                  # -7 to 7
thigh = 0                       # -7 to 7
calf = 0                        # -7 to 7
leg_length = 0                  # -7 to 7
head_length = 0                 # -7 to 7
head_width = 0                  # -7 to 7
head_depth = 0                  # -7 to 7

[appearance.strip]
sleeves = "short"               # "seasonal", "short", "long"
inners = "off"                  # "off", "normal", "turtleneck"
socks = "standard"              # "standard", "long", "short"
undershorts = "off"             # "off", "short", "winter_long" (none in summer, long in winter), "short_winter_long" (short in summer, long in winter)
untucked = true                 # true, false
ankle_taping = false            # true, false
wrist_taping = "off"            # "off", "right", "left", "both"
wrist_tape_color_left = 0       # 0 to 7
wrist_tape_color_right = 0      # 0 to 7
spectacles = 0                  # 0 none, 1 rectangle rimless, 2 rectangle half frame, 3 rectangle full frame, 4 oval rimless, 5 oval half frame, 6 oval full frame, 7 round full frame
spectacles_color = 0            # 0 white, 1 black, 2 red, 3 blue, 4 yellow, 5 green, 6 pink, 7 turquoise
gloves = false                  # true, false (outfield player gloves)
gloves_color = 0                # 0 to 7

[appearance.motion]
hunching_dribbling = 1          # 1 to 3 (PES 20 and 21: 1 to 5)
hunching_running = 1            # 1 to 3 (PES 20 and 21: 1 to 5)
arm_movement_dribbling = 1      # 1 to 8 (PES 20 and 21: 1 to 10)
arm_movement_running = 1        # 1 to 8 (PES 20 and 21: 1 to 10)
corner_kick = 1                 # 1 to 6 (PES 20 and 21: 1 to 10)
free_kick = 1                   # 1 to 16 (PES 20 and 21: 1 to 20)
penalty_kick = 1                # 1 to 4 (PES 20 and 21: 1 to 7)
dribbling = 0                   # 0 to 3, PES 20 and 21 only
goal_celebration_1 = 0          # 0 none, 1 to 122 (PES 20 and 21: 1 to 162)
goal_celebration_2 = 0          # 0 none, 1 to 122 (PES 20 and 21: 1 to 162)

[appearance.face]
cheek_type = 0                  # 0 to 3
forehead_type = 0               # 0 to 5
facial_hair_type = 0            # 0 to 12 (PES 20 and 21: 0 to 19)
laughter_lines_type = 0         # 0 to 4
upper_eyelid_type = 0           # 0 to 6 (PES 20 and 21: 0 to 7)
lower_eyelid_type = 0           # 0 to 2 (PES 20 and 21: 0 to 6)
eyebrow_type = 0                # 0 to 5 (PES 20 and 21: 0 to 7)
neck_line_type = 0              # 0 to 2 (PES 20 and 21: 0 to 3)
nose_type = 0                   # 0 to 6 (PES 20 and 21: 0 to 7)
upper_lip_type = 0              # 0 to 3 (PES 20 and 21: 0 to 4)
lower_lip_type = 0              # 0 to 2 (PES 20 and 21: 0 to 4)

# Bits of the game's record no tool has decoded yet (hair, face sliders, ...), kept so a
# player's look survives; one table per game version, named by bit position. Leave them
# as generated.
[appearance.unknown.pes21]
bit_2002 = false
bits_2096_2143 = "AAAAAAAA"
```

Where the ranges come from, so a wrong one can be traced: the boots/gloves range is the stock
band below the compiler's first team block (`player_folders.md` "Assigns IDs automatically"); the colour, spectacle, sleeve, inner,
sock, undershort, shirttail and wrist-taping lists are the reference save editor's combo boxes
(stored value = list position; wrist taping 1 = right, 2 = left, 3 = both, matching the two taping
bits); the physique numbers are the editor's spin ranges (stored value = number + 7); the motion
ranges are its spin ranges per version (1-based ones are stored minus 1, celebrations as is); the
face-type maxima are the caps the two legacy converters apply when writing a PES 16 and a PES 21
save. The 17 to 19 face caps are unmeasured and assumed equal to PES 16's; PES 15's face caps are
assumed equal to 16's too. The parser checks a value against the **widest** range of a key (a PES
16 file may legitimately be compiled for PES 21); the per-version narrowing is the compiler's
finding at compile time, when the target version is known. Strings are matched exactly (lower
case, as listed). Height and weight are not keys: they are the player record's own fields, not the
appearance block's, and height affects gameplay, so both belong to the manager's tactical side
(the save editor), and weight leaves with height for consistency.

The unknown bits follow four rules:

- **Named by position.** `bit_N` is one undecoded bit at bit offset `N` of the player record
  (the numbering of `pes_savefile`'s schema tables); `bits_A_B` is the run of adjacent undecoded
  bits from `A` to `B` inclusive, packed from bit `A` upward into bytes, little-endian like the
  record, then base64. Which bits are undecoded is `pes_savefile`'s completeness test's
  complement, per version; the positions in the template above are illustrative.
- **One table per version.** `[appearance.unknown.pes21]` applies only to a PES 21 compile; a
  compile for another version leaves those bits at their defaults and reports the table it
  could not use (`settings_unknown_other_version`, I). Bits mean nothing across versions, so they
  are never converted. A template for a new player folder, which has no player's bits to carry,
  holds no `[appearance.unknown]` table; its absent bits are 0.
- **Default 0.** An absent bit or run is 0. Whether an all-zero default renders well is unchecked:
  the first in-game check of a compile from a file without the table settles it, and if zeros
  misbehave the default becomes the base database template row's bits instead.
- **Not edited by hand.** The table exists so a generated file carries a player's whole look; the
  Save editor and the Export upgrader write it, and the Player aesthetics editor keeps it when it
  rewrites a file. When a bit is decoded it moves to a named key and leaves the table.

The four `undershorts` labels name the summer/winter pair the game stores (the reference editor
shows them as "S: Off / W: Off", "S: Short / W: Short", "S: Off / W: Long", "S: Short / W: Long").

Name-writing rules, in full — **not writing is the default**; writing is opt-in:

1. `name` absent → no name is written, ever. The savefile name stays whatever the save editor set.
   This is also what makes multi-mapped `players.txt` folders work as **generic model folders**:
   several players can share one folder's models without being renamed.
2. `name = true` → the name is derived from the folder (the name part of a numbered folder name, or
   the whole folder name under `players.txt`) and written.
3. `name = "..."` → the string is written as-is. This is also the only way to write a name carrying
   the savefile's **name colour codes** (`\x11c` + 8 hex digits, see the pes_savefile plan's "Name
   colour codes") — control characters cannot appear in a folder name, so a derived name (`true`)
   is always plain. Cup organizers colour names freely (not only medal players'), so tools that
   generate `settings.toml` from a savefile (the Export upgrader, the save editor) must emit the
   explicit string whenever the savefile name carries codes, or the next compile would strip them.
4. With `name = true` or a string in a folder mapped to multiple players via `players.txt`, the same
   name is applied to all of them, with a warning (`settings_toml_name_shared`).

`shirt_name` is a string or absent: absent writes nothing, a string is written as is (checked
against the version's shirt-name length and character set, `shirt_name_from`'s limits). There is
no `true`: a shirt name rarely repeats the player name, and a derived default would be one more
thing to read. A shared folder applies it to every mapped player, like `name`.

On Fox the names are the only part of this file that reaches the savefile, and only through the
**aesthetics patch**: an autopilot team (an aesthetics export by a caretaker, a tactical export by the
council, often written before the destination team was known, with names like "Gold striker")
needs its names written over the tactical export's, and the patch makes that automatic. The patch
is applied whole, with no team picker: a managed team's export leaves `name` and `shirt_name` out,
so its players keep the names their manager set in the save editor.

Compile-time flow: resolve boots/gloves IDs from the models and link files present (a broken link is
caught as `link_target_missing` — a check impossible in the current split-tool workflow), falling
back per category to the authored stock `boots_id`/`gloves_id` → parse accepted TOML → compile →
resolve the settings and the IDs of content that was actually written, then per engine:

- **Fox:** each compiled player's `PlayerAppearance.bin`, `BootsList.bin` and `GloveList.bin` rows
  are built from the resolved settings and written into the output CPK with every other player's
  installed row (see "Bins accumulation" in the [Team compiler plan](../team_compiler/pipeline.md));
  the resolved names go into the **aesthetics patch** beside the CPK, which then holds names and
  shirt names only.
- **Pre-Fox:** everything resolved goes into the aesthetics patch.

If a local savefile is configured, the compiler applies that patch to it through `pes_savefile`
(decrypt, apply, re-encrypt, `.bak`); a Fox names write leaves the player's appearance id as it
is, so a stripped player stays stripped. The patch is what a DLC builder hands to the savefile
builder, who applies it in the save editor; the compiler has no other savefile write path (see
"Post-processing" in the [Team compiler plan](../team_compiler/pipeline.md)).

The save editor view can also **generate** these TOML files from an existing savefile, giving teams
a migration path from the current workflow. Generation (`PlayerSettings::from_player`, shared with
the Export upgrader) writes every key the save's version holds, the save's unknown bits included,
so a generated file reproduces the player as he is, whatever the defaults; it emits
`boots_id`/`gloves_id` only for a stored ID from 1 to 100: 0 (the default boots, normal hands)
stays unset, which is its default, and an ID above 100 is custom content, which the upgrader
migrates into a folder or link (the old per-team blocks start at 101 too) and the save editor
leaves out. It writes `name` per the Export upgrader's rule (`true` when the folder's name part
equals the save's name, the explicit string otherwise; the save editor, generating without a
folder, writes the string), `shirt_name` as the save's string, and no `height` or `weight`.

---
