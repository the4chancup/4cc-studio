# In-game check harness

Scripts for checking compiled output in the game itself: put a compiled CPK where the game
loads it, start the game, drive its menus, take screenshots, close it, and put the game back as
it was. Each step is one command, so the person (or agent) running the check looks at a
screenshot before choosing the next inputs.

Edit mode is enough to check how a player looks: players look the same there as in a match,
goalkeepers included. Only an outfielder's second kit onward needs a match, since Edit mode
shows kit 1 alone.

| Script | Does |
|---|---|
| `game.py` | `launch`, `pad-serve`/`pad`/`pad-stop`, `obs-shot`, `keys`, `shot`, `status`, `close` for one game (install paths in its `GAMES`) |
| `burst.py` | a series of screenshots plus contact sheets, for a cutscene such as the walkout |
| `install.py` | `install` a compiled CPK into the game's test slot, `revert` it (slot and hash in its `TEST_SLOTS`) |

Each script's docstring has its exact usage; running one without arguments prints it.

## Pad and OBS, or keys and screen

Two ways to drive the game. **The pad and OBS** (`pad`, `obs-shot`) leave the user's keyboard
alone: the game reads a virtual Xbox 360 pad in the background, and OBS's Game Capture source
draws the game even when other windows cover it, so the user keeps working while a check runs.
**Keys and the screen** (`keys`, `shot`, and `burst.py`) bring the game window to the front
before each command, since the game reads keys only from the foreground: they take the user's
keyboard. Use the pad and OBS; keys and the screen are the fallback when OBS is not running.

## Requirements

- Windows (the window and process calls are Win32), Python 3.10 or later.
- The games installed where `GAMES` in `game.py` says (`E:/PES2017`, `E:/PES2021`), each with
  Sider at the path given there. `game.py launch` starts Sider first, then the game, not elevated.
- For the pad: the ViGEmBus driver and `pip install vgamepad`, and the game's settings tool set
  to XInput controllers (with DirectInput and no pad assigned, the game ignores the pad).
- For `obs-shot`: OBS running with a Game Capture (or Window Capture) source of the game in its
  active scene, its WebSocket server on (Tools > WebSocket Server Settings, port 4455,
  authentication off: `OBS` in `game.py`), and `pip install obs-websocket-py`.
- For keys and the screen: `pip install mss Pillow pydirectinput` (`pydirectinput` sends
  DirectInput scan codes, the only keys the games read).
- `install.py` knows PES 17's test slot only: `download/4cc_90_test.cpk`, the last entry of its
  DpFileList, so it overrides every other CPK. Another game needs its own `TEST_SLOTS` entry:
  the slot's file and the SHA-256 of the file installed there.

## One check, end to end (PES 17)

Run every command from the repository root. Screenshots go under `.tmp/` (gitignored). Below,
`pad 17 ...` is short for `python scripts/ingame/game.py pad 17 ...`.

1. Compile the export without installing it, as the test slot's CPK so the compile reads the
   install's kit configs and colors: `cpk_name = '4cc_90_test'` in the `[team-compiler]`
   settings, then `4cc-studio team-compiler compile --no-deploy --export <export folder>`. The
   CPK lands in the output folder (the `output_folder_path` setting, beside the executable).
2. With the game closed, put it into the test slot:
   `python scripts/ingame/install.py install 17 <output folder>/4cc_90_test.cpk`.
   The first install keeps the slot's own file as `.tmp/ingame/pes17/4cc_90_test.cpk.original`.
3. Plug the pad in for the session, in the background: `python scripts/ingame/game.py pad-serve`.
   It stays plugged in until `pad-stop`; a pad plugged in per command is never taken by the game.
4. `python scripts/ingame/game.py launch 17`.
5. Drive to what the check looks at, with
   `python scripts/ingame/game.py obs-shot .tmp/ingame/step.png 512` after each move to see where
   the menus went. Menu positions depend on the save and the team, so these sequences are where to
   start, not fixed (team /a/ of the 4chan Cup on the maintainer's save, 2026-10-10):
   - To the main menu: `pad 17 wait:12 a wait:3 a wait:14`. The left stick moves the main
     menu's tabs, the d-pad its tiles: `pad 17 ls-left wait:1 ls-left wait:2` lands on EXTRAS,
     whose cursor starts on APPLY LIVE UPDATE; EDIT is one tile up.
   - **Edit mode, a player's appearance**: `pad 17 up wait:1 a wait:6` (Edit), `a wait:4`
     (Ponies), `down wait:1 a wait:4` (Edit Pony), `a wait:3` (4chan Cup), `down wait:1 a wait:4`
     (/a/, the second team), then on the roster's first player `a wait:4` and
     `down down down down down wait:1 a wait:5` (Appearance). `b wait:2 b wait:3` returns to the
     roster, `down` moves to the next player. `obs-shot` with no width gives the full 1920x1080
     frame to crop.
   - **A match's walkout** (keys; the enters accept the last match's teams, which the game
     remembers only when that match was quit from the pause menu):
     `keys 17 wait:12 enter wait:3 enter wait:12 enter wait:5 enter wait:5 enter wait:2 enter
     wait:3 enter wait:2 enter wait:2 enter wait:3`, then `keys 17 wait:4` and at once
     `python scripts/ingame/burst.py 17 48 walkout 1.2` for 48 frames 1.2 s apart
     (`.tmp/ingame/walkout_NN.png`, with `walkout_sheetN.png` contact sheets of 16 frames each).
6. `python scripts/ingame/game.py close 17` (ends the game and Sider), then
   `python scripts/ingame/game.py pad-stop` (unplugs the pad).
7. `python scripts/ingame/install.py revert 17` puts the slot's own file back. The kept copy
   stays in `.tmp/ingame/pes17/` for the next check.

## What it never touches

- No save: nothing here reads or writes the game's save folder. Leaving Edit mode with a change
  confirmed makes the game write its save itself, so look, do not confirm.
- No `DpFileList.bin`, and no CPK but the test slot. `install.py` writes the slot alone (through
  a `.partial` file beside it, renamed into place once its hash matches) and refuses when the
  game runs or the slot holds neither its original nor the CPK it installed last.
- No game executable or Sider file: `game.py` only starts and ends them.
