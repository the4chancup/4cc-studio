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
  python scripts/ingame/game.py pad-serve                plug in a virtual Xbox 360 pad and keep
                                                         it plugged in (run it in the background)
  python scripts/ingame/game.py pad GAME INPUT [INPUT ...]   press pad inputs in order through
                                                         `pad-serve`'s pad (`PAD_BUTTONS`,
                                                         `PAD_STICKS` names; "wait:N";
                                                         "hold:INPUT:N")
  python scripts/ingame/game.py pad-stop                 unplug the pad and end `pad-serve`
  python scripts/ingame/game.py obs-shot OUT.png [WIDTH]  OBS's frame of its program scene
                                                         (WIDTH scales it, height in proportion)
  python scripts/ingame/game.py status GAME              is the game running, where is its window
  python scripts/ingame/game.py close GAME               terminate the game process (and Sider's)

GAME is a key of `GAMES` below (17 or 21), the one place the install paths are set.
Keys are DirectInput scan codes (pydirectinput), which the game reads only from the foreground,
so the window is brought to the front before keys or a shot. `pad` and `obs-shot` leave the
focus where it is: the game reads a virtual pad (`vgamepad`, over the ViGEmBus driver) in the
background, and OBS's Game Capture source draws the game even under other windows, so a check
driven by them does not take the user's keyboard after `launch`. Nothing here touches a save or
a game file. Needs Windows, `mss` and `Pillow` (for `shot`), `pydirectinput` (for `keys`),
`vgamepad` (for `pad`) and `obs-websocket-py` with OBS's WebSocket server on (for `obs-shot`).
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
COMMANDS = ("launch", "shot", "keys", "pad", "status", "close")
# OBS's WebSocket server (Tools > WebSocket Server Settings), authentication off.
OBS = {"host": "localhost", "port": 4455, "password": ""}
# Pad inputs by name: buttons, and stick directions as (stick, x, y) pushed fully. The menus
# move with the d-pad in lists and the left stick in the top menus' tabs; the right stick turns
# a player in Edit mode. Up is y -1.0, as the maintainer's ATF bot
# (`Tools_4cc/4cc-aes-atf-bot`, `helpers.press_left_analog`) drives these menus.
PAD_BUTTONS = {
    "a": "A", "b": "B", "x": "X", "y": "Y", "start": "START", "back": "BACK",
    "lb": "LEFT_SHOULDER", "rb": "RIGHT_SHOULDER",
    "up": "DPAD_UP", "down": "DPAD_DOWN", "left": "DPAD_LEFT", "right": "DPAD_RIGHT",
}
PAD_STICKS = {
    "ls-up": ("left", 0.0, -1.0), "ls-down": ("left", 0.0, 1.0),
    "ls-left": ("left", -1.0, 0.0), "ls-right": ("left", 1.0, 0.0),
    "rs-up": ("right", 0.0, -1.0), "rs-down": ("right", 0.0, 1.0),
    "rs-left": ("right", -1.0, 0.0), "rs-right": ("right", 1.0, 0.0),
}
# How long a tap holds an input, and the pause after it: the games miss shorter taps.
PAD_TAP_SECONDS = 0.2
PAD_GAP_SECONDS = 0.3
# A pad the driver has just plugged in is not read at once; the first inputs are lost without
# this wait.
PAD_CONNECT_SECONDS = 2.0
# The local port `pad-serve` listens on for `pad`'s input lists.
PAD_PORT = 47017
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


def pad_set(pad, name: str, pressed: bool) -> None:
    """Pushes (or releases) the pad input `name`, one of `PAD_BUTTONS` or `PAD_STICKS`."""
    import vgamepad as vg
    if name in PAD_BUTTONS:
        button = getattr(vg.XUSB_BUTTON, f"XUSB_GAMEPAD_{PAD_BUTTONS[name]}")
        if pressed:
            pad.press_button(button=button)
        else:
            pad.release_button(button=button)
    elif name in PAD_STICKS:
        stick, x, y = PAD_STICKS[name]
        if not pressed:
            x, y = 0.0, 0.0
        if stick == "left":
            pad.left_joystick_float(x_value_float=x, y_value_float=y)
        else:
            pad.right_joystick_float(x_value_float=x, y_value_float=y)
    else:
        raise SystemExit(f"unknown pad input {name!r}")
    pad.update()


def check_pad_inputs(inputs: list[str]) -> None:
    for item in inputs:
        name = item.split(":")[1] if item.startswith("hold:") else item
        if not item.startswith("wait:") and name not in PAD_BUTTONS and name not in PAD_STICKS:
            raise SystemExit(f"unknown pad input {name!r}")


def cmd_pad_serve() -> None:
    """Plugs in one virtual pad and keeps it plugged in until `pad-stop`, playing each input
    list `pad` sends. One pad for the whole session, not one per `pad` call: a pad plugged in
    and out at every call is never taken by the game as its controller."""
    import socket
    import vgamepad as vg
    pad = vg.VX360Gamepad()
    time.sleep(PAD_CONNECT_SECONDS)
    with socket.create_server(("127.0.0.1", PAD_PORT)) as server:
        print(f"pad plugged in; serving on port {PAD_PORT}", flush=True)
        while True:
            connection, _ = server.accept()
            with connection:
                line = connection.makefile("r", encoding="utf-8").readline().split()
                if line == ["stop"]:
                    connection.sendall(b"stopped\n")
                    return
                play_pad_inputs(pad, line)
                connection.sendall(b"sent\n")


def send_to_pad_server(words: list[str]) -> str:
    import socket
    try:
        with socket.create_connection(("127.0.0.1", PAD_PORT), timeout=5) as connection:
            # Inputs may wait for a while; the reply comes when they are all played.
            connection.settimeout(None)
            connection.sendall((" ".join(words) + "\n").encode("utf-8"))
            return connection.makefile("r", encoding="utf-8").readline().strip()
    except ConnectionRefusedError:
        raise SystemExit("no pad server: start `game.py pad-serve` first (in the background)")


def cmd_pad(game: dict[str, str], inputs: list[str]) -> None:
    check_pad_inputs(inputs)
    window_or_die(game)
    send_to_pad_server(inputs)
    print(f"sent: {' '.join(inputs)}")


def play_pad_inputs(pad, inputs: list[str]) -> None:
    for item in inputs:
        if item.startswith("wait:"):
            time.sleep(float(item[5:]))
            continue
        if item.startswith("hold:"):
            _, name, seconds = item.split(":")
            hold = float(seconds)
        else:
            name, hold = item, PAD_TAP_SECONDS
        pad_set(pad, name, True)
        time.sleep(hold)
        pad_set(pad, name, False)
        time.sleep(PAD_GAP_SECONDS)


def cmd_obs_shot(out: str, width: int | None) -> None:
    import base64
    from obswebsocket import obsws, requests
    ws = obsws(OBS["host"], OBS["port"], OBS["password"])
    ws.connect()
    try:
        scene = ws.call(requests.GetCurrentProgramScene()).datain["currentProgramSceneName"]
        fields = {"sourceName": scene, "imageFormat": "png"}
        if width:
            fields["imageWidth"] = width
        data = ws.call(requests.GetSourceScreenshot(**fields)).datain["imageData"]
    finally:
        ws.disconnect()
    png = base64.b64decode(data.split(",", 1)[1])
    temporary = out + ".tmp.png"
    with open(temporary, "wb") as handle:
        handle.write(png)
    os.replace(temporary, out)
    print(f"{out}: {len(png)} bytes, scene {scene!r}")


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
    # These name no game: OBS captures whatever its scene shows, and the pad is the system's.
    if len(args) in (2, 3) and args[0] == "obs-shot":
        cmd_obs_shot(args[1], int(args[2]) if len(args) > 2 else None)
        return 0
    if args == ["pad-serve"]:
        cmd_pad_serve()
        return 0
    if args == ["pad-stop"]:
        print(send_to_pad_server(["stop"]))
        return 0
    if len(args) < 2 or args[0] not in COMMANDS or args[1] not in GAMES:
        print(__doc__, file=sys.stderr)
        return 2
    command, game = args[0], GAMES[args[1]]
    if command in ("shot", "keys", "pad") and len(args) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    if command == "launch":
        cmd_launch(game)
    elif command == "shot":
        cmd_shot(game, args[2], float(args[3]) if len(args) > 3 else 1.0)
    elif command == "keys":
        cmd_keys(game, args[2:])
    elif command == "pad":
        cmd_pad(game, args[2:])
    elif command == "status":
        cmd_status(game)
    else:
        cmd_close(game)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
