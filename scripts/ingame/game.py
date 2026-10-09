"""In-game test harness: launch a PES install, screenshot its window, send it keys, close it.

Each command is one process call, so whoever drives a check reads a screenshot between steps.
`README.md` beside this file has the procedure of a whole check.

  python scripts/ingame/game.py launch GAME              start Sider, then the game (not
                                                         elevated); waits up to 120 s for its window
  python scripts/ingame/game.py shot GAME OUT.png [SCALE]   screenshot of the game's client area
                                                         (SCALE 0.5 halves it)
  python scripts/ingame/game.py keys GAME KEY [KEY ...]  press keys in order (pydirectinput names;
                                                         "wait:N" sleeps N seconds; "hold:KEY:N"
                                                         holds KEY N seconds)
  python scripts/ingame/game.py status GAME              is the game running, where is its window
  python scripts/ingame/game.py close GAME               terminate the game process (and Sider's)

GAME is a key of `GAMES` below (17 or 21), the one place the install paths are set.
Keys are DirectInput scan codes (pydirectinput), the only kind the games read. The window is
brought to the front before keys or a shot. Nothing here touches a save or a game file.
Needs Windows, `mss` and `Pillow` (for `shot`) and `pydirectinput` (for `keys`).
"""

import ctypes
import ctypes.wintypes as wt
import os
import subprocess
import sys
import time

# Per game: the install folder, the game's executable in it, Sider's executable relative to it,
# and the process name tasklist shows for the running game.
GAMES = {
    "17": {"dir": "E:/PES2017", "exe": "PES2017.exe", "sider": "Sider/sider.exe", "proc": "pes2017.exe"},
    "21": {"dir": "E:/PES2021", "exe": "PES2021.exe", "sider": "sider/sider.exe", "proc": "pes2021.exe"},
}
COMMANDS = ("launch", "shot", "keys", "status", "close")
user32 = ctypes.windll.user32


def pids_of(proc_name: str) -> list[int]:
    out = subprocess.run(["tasklist", "/FI", f"IMAGENAME eq {proc_name}", "/FO", "CSV", "/NH"],
                         capture_output=True, text=True, check=True).stdout
    pids = []
    for line in out.splitlines():
        parts = [p.strip('"') for p in line.split('","')]
        if len(parts) >= 2 and parts[0].lower() == proc_name.lower():
            pids.append(int(parts[1]))
    return pids


def main_window(pids: list[int]) -> tuple[int, str] | None:
    """The first visible top-level window owned by one of `pids`."""
    found = []
    proto = ctypes.WINFUNCTYPE(ctypes.c_bool, wt.HWND, wt.LPARAM)

    def callback(hwnd, _):
        if not user32.IsWindowVisible(hwnd):
            return True
        pid = wt.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value in pids:
            length = user32.GetWindowTextLengthW(hwnd)
            title = ctypes.create_unicode_buffer(length + 1)
            user32.GetWindowTextW(hwnd, title, length + 1)
            rect = wt.RECT()
            user32.GetClientRect(hwnd, ctypes.byref(rect))
            if rect.right > 100 and rect.bottom > 100:
                found.append((hwnd, title.value))
        return True

    user32.EnumWindows(proto(callback), 0)
    return found[0] if found else None


def client_rect(hwnd: int) -> dict[str, int]:
    rect = wt.RECT()
    user32.GetClientRect(hwnd, ctypes.byref(rect))
    origin = wt.POINT(0, 0)
    user32.ClientToScreen(hwnd, ctypes.byref(origin))
    return {"left": origin.x, "top": origin.y, "width": rect.right, "height": rect.bottom}


def bring_front(hwnd: int) -> None:
    """Makes `hwnd` the foreground window. A process that is not in the foreground may not
    steal focus, so a tap of Alt (the documented loophole) precedes the call; the result is
    checked, since a screenshot of the wrong window would read as a wrong game state."""
    user32.ShowWindow(hwnd, 9)  # SW_RESTORE
    for _ in range(3):
        if user32.GetForegroundWindow() == hwnd:
            break
        user32.keybd_event(0x12, 0, 0, 0)  # Alt down
        user32.SetForegroundWindow(hwnd)
        user32.keybd_event(0x12, 0, 2, 0)  # Alt up (KEYEVENTF_KEYUP)
        time.sleep(0.4)
    if user32.GetForegroundWindow() != hwnd:
        raise SystemExit("the game window could not be brought to the front")
    time.sleep(0.3)


def window_or_die(game: dict[str, str]) -> tuple[int, str]:
    pids = pids_of(game["proc"])
    if not pids:
        raise SystemExit(f"{game['proc']} is not running")
    win = main_window(pids)
    if not win:
        raise SystemExit(f"{game['proc']} (pid {pids}) has no visible window yet")
    return win


def cmd_launch(game: dict[str, str]) -> None:
    if pids_of(game["proc"]):
        raise SystemExit(f"{game['proc']} is already running")
    cwd = game["dir"]
    sider = f"{cwd}/{game['sider']}"
    if not pids_of("sider.exe"):
        subprocess.Popen([sider], cwd=os.path.dirname(sider), creationflags=subprocess.DETACHED_PROCESS)
        time.sleep(3)
    subprocess.Popen([f"{cwd}/{game['exe']}"], cwd=cwd, creationflags=subprocess.DETACHED_PROCESS)
    for _ in range(120):
        time.sleep(1)
        pids = pids_of(game["proc"])
        if not pids:
            continue
        win = main_window(pids)
        if win:
            print(f"window {win[0]:#x} {win[1]!r} {client_rect(win[0])}")
            return
    raise SystemExit("no game window after 120 s")


def cmd_shot(game: dict[str, str], out: str, scale: float) -> None:
    import mss
    from PIL import Image
    hwnd, _ = window_or_die(game)
    bring_front(hwnd)
    rect = client_rect(hwnd)
    with mss.MSS() as sct:
        grab = sct.grab(rect)
        image = Image.frombytes("RGB", grab.size, grab.bgra, "raw", "BGRX")
    if scale != 1.0:
        image = image.resize((int(image.width * scale), int(image.height * scale)), Image.LANCZOS)
    temporary = out + ".tmp.png"
    image.save(temporary, "PNG")
    os.replace(temporary, out)
    print(f"{out}: {image.width}x{image.height} from {rect}")


def cmd_keys(game: dict[str, str], keys: list[str]) -> None:
    import pydirectinput
    pydirectinput.PAUSE = 0.15
    hwnd, _ = window_or_die(game)
    bring_front(hwnd)
    for key in keys:
        if key.startswith("wait:"):
            time.sleep(float(key[5:]))
        elif key.startswith("hold:"):
            _, name, seconds = key.split(":")
            pydirectinput.keyDown(name)
            time.sleep(float(seconds))
            pydirectinput.keyUp(name)
        else:
            pydirectinput.press(key)
    print(f"sent: {' '.join(keys)}")


def cmd_status(game: dict[str, str]) -> None:
    pids = pids_of(game["proc"])
    print(f"{game['proc']}: pids {pids}; sider.exe: {pids_of('sider.exe')}")
    if pids:
        win = main_window(pids)
        print(f"window: {win and (hex(win[0]), win[1], client_rect(win[0]))}")


def cmd_close(game: dict[str, str]) -> None:
    for name in (game["proc"], "sider.exe"):
        for pid in pids_of(name):
            subprocess.run(["taskkill", "/PID", str(pid), "/F"], capture_output=True, check=False)
            print(f"killed {name} {pid}")
    time.sleep(2)
    if pids_of(game["proc"]):
        raise SystemExit(f"{game['proc']} still running")


def main(args: list[str]) -> int:
    if len(args) < 2 or args[0] not in COMMANDS or args[1] not in GAMES:
        print(__doc__, file=sys.stderr)
        return 2
    command, game = args[0], GAMES[args[1]]
    if command in ("shot", "keys") and len(args) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    if command == "launch":
        cmd_launch(game)
    elif command == "shot":
        cmd_shot(game, args[2], float(args[3]) if len(args) > 3 else 1.0)
    elif command == "keys":
        cmd_keys(game, args[2:])
    elif command == "status":
        cmd_status(game)
    else:
        cmd_close(game)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
