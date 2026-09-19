# 4cc Studio — Aesthetics export plan: settings.toml

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player settings in exports (settings.toml)

Savefile settings that currently live only in the savefile move into the export as TOML, written to
the savefile at compile time. **Everything in the file is optional** — including the file itself: a
player folder with only models is valid, and an absent authored key means "leave that savefile
setting untouched"; model-derived ID assignments and FPC directives are separate inputs.

**The file can express every known aesthetic setting the savefile holds.** Its schema is
`pes_savefile`'s `PlayerSettings`: all of the decoded player appearance record — physique, strip
style, wrist-tape and spectacle colours, skin and iris colour, the motion block, the eleven
ingame-face feature types — except the two compiler-owned groups (boots/gloves IDs, edit flags).
"Known" is the one qualifier: the ingame-face run also carries bits no tool has decoded (hair,
face-slider positions, colours beyond skin and iris); `pes_savefile` carries them byte for byte
through every edit and transplant, but nothing can author them until they are measured (see
"Player settings model" in the [Savefile plan](../pes_savefile/model.md), which also holds the
completeness test). Completeness matters here because this file is the **only** route by which a
team's aesthetics reach the official save: the compiler resolves it into an aesthetics patch, the
save editor applies the patch, and the editor's own appearance fields are read-only by default
(see "Read-only aesthetics" in the [Save editor plan](../save_editor.md)). The **template** — what
the Export upgrader and the save editor generate, and what a new player folder starts from —
therefore lists every key, one per line, each followed on the same line by the comment that gives
its range: set ones with their value, unset ones as commented lines, so a team can see what is
authorable without any other document. The block below is the complete key table; `to_toml`
emits it in this order and with these comments.

**Boots/gloves IDs are not authored settings.** Export `settings.toml` neither accepts nor generates
boots/gloves ID keys. Local models and shared link files determine the compiler's player-exclusive
or shared assignments, using the existing per-target rules; users never copy numeric IDs into this
file. Pre-Fox local models embedded in face XML still need no standalone ID, as described above.

```toml
# settings.toml, inside a player folder. Every key is optional: an absent key leaves
# that savefile setting untouched. A commented key shows what can be set and its range.
# The file never references models: what a player wears is decided by the folder
# contents (models present, link files pointing at shared folders).

# true = derive from the folder name ("15 - Snuffy" gives "Snuffy"; the whole folder
# name for players.txt-mapped folders); "text" = write as is; absent = leave untouched.
name = "Snuffy"

[appearance]
skin_color = 1                  # 0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)
iris_color = 0                  # 0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet

[appearance.physique]
height = 180                    # cm
weight = 75                     # kg
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
```

Where the ranges come from, so a wrong one can be traced: the colour, spectacle, sleeve, inner,
sock, undershort, shirttail and wrist-taping lists are the reference save editor's combo boxes
(stored value = list position; wrist taping 1 = right, 2 = left, 3 = both, matching the two taping
bits); the physique numbers are the editor's spin ranges (stored value = number + 7); the motion
ranges are its spin ranges per version (1-based ones are stored minus 1, celebrations as is); the
face-type maxima are the caps the two legacy converters apply when writing a PES 16 and a PES 21
save. The 17 to 19 face caps are unmeasured and assumed equal to PES 16's; PES 15's face caps are
assumed equal to 16's too. The parser checks a value against the **widest** range of a key (a PES
16 file may legitimately be compiled for PES 21); the per-version narrowing is the compiler's
finding at compile time, when the target version is known. Strings are matched exactly (lower
case, as listed). Height and weight are the player record's own fields, not the appearance block's,
and are the only two keys here that also affect gameplay; they stay because the reference editor's
Appearance tab owns them and teams set them for looks.

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

Compile-time flow: resolve boots/gloves IDs from the models and link files present (a broken link is
caught as `link_target_missing` — a check impossible in the current split-tool workflow) → parse
accepted TOML → compile → resolve the settings and the IDs of content that was actually written into
the **aesthetics patch** written beside the CPK → if a local savefile is configured, apply that
patch to it through `pes_savefile` (decrypt, apply, re-encrypt, `.bak`). The patch is what a DLC
builder hands to the savefile builder, who applies it in the save editor; the compiler has no
other savefile write path (see "Post-processing" in the [Team compiler plan](../team_compiler/pipeline.md)).

The save editor view can also **generate** these TOML files from an existing savefile, giving teams
a migration path from the current workflow.

---
