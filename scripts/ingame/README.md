# In-game check harness

Scripts for checking compiled output in the game itself: put a compiled CPK where the game
loads it, start the game, drive its menus with keys, take screenshots, close it, and put the
game back as it was. Each step is one command, so the person (or agent) running the check looks
at a screenshot before choosing the next keys.

Edit mode is enough to check how a player looks: players look the same there as in a match,
goalkeepers included. Only an outfielder's second kit onward needs a match, since Edit mode
shows kit 1 alone.

| Script | Does |
|---|---|
| `game.py` | `launch`, `shot`, `keys`, `status`, `close` for one game (install paths in its `GAMES`) |
| `burst.py` | a series of screenshots plus contact sheets, for a cutscene such as the walkout |
| `install.py` | `install` a compiled CPK into the game's test slot, `revert` it (slot and hash in its `TEST_SLOTS`) |

Each script's docstring has its exact usage; running one without arguments prints it.

## Requirements

- Windows (the window and process calls are Win32), Python 3.10 or later.
- `pip install mss Pillow pydirectinput`: `mss` and `Pillow` for screenshots, `pydirectinput`
  for keys (DirectInput scan codes, the only kind the games read).
- The games installed where `GAMES` in `game.py` says (`E:/PES2017`, `E:/PES2021`), each with
  Sider at the path given there. `game.py launch` starts Sider first, then the game, not elevated.
- `install.py` knows PES 17's test slot only: `download/4cc_90_test.cpk`, the last entry of its
  DpFileList, so it overrides every other CPK. Another game needs its own `TEST_SLOTS` entry:
  the slot's file and the SHA-256 of the file installed there.

## One check, end to end (PES 17)

Run every command from the repository root. Screenshots go under `.tmp/` (gitignored).

1. Compile the export without installing it:
   `4cc-studio team_compiler compile --no-deploy --export <export folder>`. The CPKs land in
   the output folder (the `output_folder_path` setting, beside the executable).
2. With the game closed, put the CPK to check into the test slot:
   `python scripts/ingame/install.py install 17 <output folder>/<the CPK to check>.cpk`.
   The first install keeps the slot's own file as `.tmp/ingame/pes17/4cc_90_test.cpk.original`.
3. `python scripts/ingame/game.py launch 17`.
4. Drive to what the check looks at (`keys 17 ...` below is short for
   `python scripts/ingame/game.py keys 17 ...`), with
   `python scripts/ingame/game.py shot 17 .tmp/ingame/step.png 0.5` after each move to see where
   the menus went. Menu positions depend on the save and the team, so these sequences, both
   starting right after the launch, are where to start, not fixed:
   - **Edit mode, a player's appearance** (team /a/ of the 4chan Cup on the maintainer's save,
     2026-10-09): to the main menu with `keys 17 wait:12 enter wait:3 enter wait:14`, then
     `keys 17 left wait:1 left wait:2 up wait:1 enter wait:6`, then
     `keys 17 enter wait:4`, `keys 17 down enter wait:4`, `keys 17 enter wait:3`,
     `keys 17 down wait:1 enter wait:4`, `keys 17 enter wait:4`, and on a player
     `keys 17 down down down down down wait:1 enter wait:5`. `esc wait:2 esc wait:3` returns to
     the roster, `down` moves to the next player.
   - **A match's walkout** (the enters accept the last match's teams, which the game
     remembers only when that match was quit from the pause menu):
     `keys 17 wait:12 enter wait:3 enter wait:12 enter wait:5 enter wait:5 enter wait:2 enter
     wait:3 enter wait:2 enter wait:2 enter wait:3`, then `keys 17 wait:4` and at once the burst
     of step 5.
5. Record what is on screen: `python scripts/ingame/game.py shot 17 .tmp/ingame/player.png` for
   one frame, or `python scripts/ingame/burst.py 17 48 walkout 1.2` for 48 frames 1.2 s apart
   (`.tmp/ingame/walkout_NN.png`, with `walkout_sheetN.png` contact sheets of 16 frames each).
6. `python scripts/ingame/game.py close 17` (ends the game and Sider).
7. `python scripts/ingame/install.py revert 17` puts the slot's own file back. The kept copy
   stays in `.tmp/ingame/pes17/` for the next check.

## What it never touches

- No save: nothing here reads or writes the game's save folder. Leaving Edit mode with a change
  confirmed makes the game write its save itself, so look, do not confirm.
- No `DpFileList.bin`, and no CPK but the test slot. `install.py` writes the slot alone (through
  a `.partial` file beside it, renamed into place once its hash matches) and refuses when the
  game runs or the slot holds neither its original nor the CPK it installed last.
- No game executable or Sider file: `game.py` only starts and ends them.
