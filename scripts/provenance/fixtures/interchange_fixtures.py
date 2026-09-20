"""Extract the pes_savefile interchange fixtures (2.17h) from the real files on the reference
machine, and print the literals the golden tests assert.

- `pes18_texport.ted`, `pes19_texport.ted`, `pes21_texport.ted`: whole real Texport files (8-15 KB,
  no personal data beyond the team's content). Decrypted here with the reference editor's XOR
  scheme to print the record ids the layout test asserts.
- `pes17_texport_head.bin` / `pes17_texport_payload_enc_head.bin` / `pes17_texport_payload.bin.zz`:
  the same slices `save_fixtures.py` cuts from a save, cut from the PES 17 `TEXPORT00000000`
  (5.6 MB, so sliced; the payload is mostly zeros and zlibs to ~25 KB).
- `pes19_squad.4ccs`: a real `.4ccs` written from the PES 19 save (tag "20a"). Decoded here with
  the ctypes mirror of `player_export` (MSVC layout) to print the offsets and values the golden
  test holds as literals.
- `pes19_tactics.4cct`: synthesized (no real file exists): the reference editor's
  `save_tactical_data` write walk over the PES 19 fixture save's team 713 tactics record, read
  with the reference's `fill_team_tactics19` walk. Instructions are stored canonical
  (`translate_adv_instruction(raw, 19)`).
"""
import ctypes
import hashlib
import struct
import sys
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from save_census import KEYS, crypt_stream, effective, file_key, section_key  # noqa: E402

OUT = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\pes_savefile\tests\fixtures")
PREFIX = 4096

TED = {  # version -> (source, key index, size, team record size, roster size, tactics size, player size, width)
    18: (r"C:\Data\Documents\KONAMI\PRO EVOLUTION SOCCER 2018\WEPES\osyearduh.ted", 0x12, 0x1FC0, 480, 164, 628, 188, 32),
    19: (r"C:\Data\Documents\KONAMI\PRO EVOLUTION SOCCER 2019\WEPES\98HU.ted", 0x13, 0x25B0, 416, 244, 628, 188, 40),
    21: (r"C:\Data\Documents\KONAMI\eFootball PES 2021 SEASON UPDATE\WEPES\98hu_vrl4_v1.ted", 0x15, 0x39E4, 588, 284, 628, 312, 40),
}
TEXPORT17 = r"C:\Data\4cc\Saves\pesXcrypter\TEXPORT00000000"
SQUAD = r"C:\Data\Documents\KONAMI\PRO EVOLUTION SOCCER 2019\292733975847239680\save\CM.4ccs"
PES19_PAYLOAD = OUT / "pes19_payload.bin.zz"
CCT_TEAM = 713


def write(path, data):
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_bytes(data)
    tmp.replace(path)


# --- Texport 18-21 -------------------------------------------------------------------------------

def xor_decrypt(data, key_index):
    key = data[0x30:0x50]
    out = bytearray(data)
    k = key_index
    for pos in range(0x50, len(data)):
        out[pos] = data[pos] ^ key[k]
        k = (k + 1) % 0x20
    return bytes(out)


def ted(version):
    src, key_index, size, team, roster, tactics, player, width = TED[version]
    data = Path(src).read_bytes()
    assert len(data) == size, (len(data), size)
    dec = xor_decrypt(data, key_index)
    team_at = 0x50
    coach_at = team_at + team
    roster_at = coach_at + 88
    tactics_at = roster_at + roster
    players_at = tactics_at + tactics + 660
    assert players_at + width * player + 12 == size
    ids = [struct.unpack_from("<I", dec, at)[0] for at in (team_at, coach_at, roster_at, tactics_at)]
    assert len(set(ids)) == 1, ids
    players = [struct.unpack_from("<I", dec, players_at + i * player)[0] for i in range(width)]
    filled = [p for p in players if p]
    assert filled == [ids[0] * 100 + 1 + i for i in range(len(filled))], filled
    assert all(p == 0 for p in players[len(filled):])
    write(OUT / f"pes{version}_texport.ted", data)
    print(f"pes{version}_texport.ted: {size} B; header {data[:0x30].hex()}")
    print(f"  team {ids[0]}; tactics @ {tactics_at:#x}; players @ {players_at:#x} x {width}, {len(filled)} filled")
    print(f"  coach block (id patched out) {dec[coach_at + 4:roster_at].hex()}")
    print(f"  tail {dec[-12:].hex()}; 660-byte block nonzero bytes {sum(1 for b in dec[tactics_at + tactics:players_at] if b)}")


# --- Texport 17 (container) ----------------------------------------------------------------------

def texport17():
    data = Path(TEXPORT17).read_bytes()
    eff = effective(KEYS[17])
    fkey = file_key(data[0:320], eff)
    hsize = 176
    header = crypt_stream(section_key(fkey, hsize), data[320 : 320 + hsize])
    assert header[0:64] == eff
    payload_size, logo_size, desc_size, serial_size = struct.unpack_from("<4I", header, 64)
    off = 320 + hsize
    desc = crypt_stream(section_key(fkey, 0), data[off : off + desc_size])
    head_end = off + desc_size
    off = head_end + logo_size
    payload_enc = data[off : off + payload_size]
    payload = crypt_stream(section_key(fkey, 2), payload_enc)
    off += payload_size
    assert off + serial_size * 2 == len(data)
    assert desc.startswith(b"Team Export Data"), desc[:20]
    assert crypt_stream(section_key(fkey, 2), payload_enc[:PREFIX]) == payload[:PREFIX]
    team_id = struct.unpack_from("<I", payload, 0x10330)[0]
    first = struct.unpack_from("<I", payload, 0x510840)[0]
    assert first == team_id * 100 + 1, (team_id, first)
    n = 0
    while struct.unpack_from("<I", payload, 0x510840 + n * 188)[0] == first + n:
        n += 1
    write(OUT / "pes17_texport_head.bin", data[:head_end])
    write(OUT / "pes17_texport_payload_enc_head.bin", payload_enc[:PREFIX])
    write(OUT / "pes17_texport_payload.bin.zz", zlib.compress(payload, 9))
    print(f"pes17_texport: head {head_end} B; payload {payload_size} logo {logo_size} desc {desc_size} serial {serial_size}")
    print(f"  description {desc.rstrip(b'\\0')!r}; team {team_id}; {n} players @ 0x510840")
    print(f"  payload md5 = {hashlib.md5(payload).hexdigest()}  zz {len(zlib.compress(payload, 9))} B")


# --- .4ccs ---------------------------------------------------------------------------------------

u8, u32, i32, b1 = ctypes.c_ubyte, ctypes.c_uint32, ctypes.c_int32, ctypes.c_bool


class PlayerExport(ctypes.Structure):
    """`player_export` (editor.h), MSVC layout: 4-byte alignment, bool one byte, wchar_t two."""

    _fields_ = (
        [("nation", u32)]
        + [(n, u8) for n in "height weight gc1 gc2 atk def gk drib mo_fk finish lowpass loftpass header form".split()]
        + [("b_edit_player", b1)]
        + [(n, u8) for n in "swerve catching clearing reflex injury".split()]
        + [("b_edit_basicset", b1)]
        + [(n, u8) for n in "body_ctrl phys_cont kick_pwr exp_pwr mo_armd".split()]
        + [("b_edit_regpos", b1)]
        + [(n, u8) for n in "age reg_pos play_style ball_ctrl ball_win weak_acc jump mo_armr mo_ck cover weak_use".split()]
        + [("play_pos", u8 * 13)]
        + [(n, u8) for n in "mo_hunchd mo_hunchr mo_pk place_kick star mo_drib tight_pos aggres play_attit".split()]
        + [(n, b1) for n in "b_edit_playpos b_edit_ability b_edit_skill".split()]
        + [("stamina", u8), ("speed", u8)]
        + [(n, b1) for n in "b_edit_style b_edit_com b_edit_motion b_base_copy".split()]
        + [("strong_foot", u8), ("strong_hand", u8)]
        + [("com_style", b1 * 7), ("play_skill", b1 * 41)]
        + [("name", ctypes.c_wchar * 61), ("shirt_name", ctypes.c_char * 21)]
        + [(n, b1) for n in "b_edit_face b_edit_hair b_edit_phys b_edit_strip".split()]
        + [("boot_id", u32), ("glove_id", u32), ("copy_id", u32)]
        + [(n, i32) for n in "neck_len neck_size shldr_hi shldr_wid chest waist arm_size arm_len thigh calf leg_len head_len head_wid head_dep".split()]
        + [(n, u8) for n in "wrist_col_l wrist_col_r wrist_tape spec_col spec_style sleeve inners socks undershorts".split()]
        + [(n, b1) for n in "untucked ankle_tape gloves".split()]
        + [(n, u8) for n in "gloves_col skin_col iris_col".split()]
    )


def squad():
    assert ctypes.sizeof(ctypes.c_wchar) == 2, "run on Windows: wchar_t is two bytes"
    size = ctypes.sizeof(PlayerExport)
    assert size == 356, size
    data = Path(SQUAD).read_bytes()
    tag, ver = data[:3], data[3:5]
    n, rem = divmod(len(data) - 5 - 80, size)
    assert rem in (0, 405), rem
    print(f"pes19_squad.4ccs: {len(data)} B; tag {tag!r}; pes {ver!r}; {n} players; tactics block: {rem == 405}")
    print("  offsets:", " ".join(f"{name}@{getattr(PlayerExport, name).offset}" for name, _ in PlayerExport._fields_))
    for i in (0, 7, 22):
        p = PlayerExport.from_buffer_copy(data, 5 + i * size)
        print(f"  [{i}] name={p.name!r} shirt={p.shirt_name!r} nation={p.nation} atk={p.atk} gk={p.gk} reg_pos={p.reg_pos} "
              f"play_style={p.play_style} star={p.star} skills={[k for k in range(41) if p.play_skill[k]]} "
              f"boot={p.boot_id} copy={p.copy_id} skin={p.skin_col} iris={p.iris_col} neck_len={p.neck_len} height={p.height}")
    numbers = struct.unpack_from("<40H", data, 5 + n * size)
    print("  numbers:", numbers)
    write(OUT / "pes19_squad.4ccs", data)


# --- .4cct (synthesized) --------------------------------------------------------------------------

def translate_adv_instruction(raw, pes_version):
    """The reference's raw (stored) -> canonical map for one version."""
    if raw <= 0x07:
        return raw
    if pes_version == 17:
        return raw if raw <= 0x0C else 0
    return {0x08: 0x0D, 0x09: 0x0E, 0x0A: 0x08, 0x0B: 0x09, 0x0C: 0x0A, 0x0D: 0x0B, 0x0E: 0x0C, 0x0F: 0x0F}[raw]


def read_tactics19(rec):
    """`fill_team_tactics19`'s walk over one 628-byte record, as a dict."""
    pos = 4
    presets = []
    for _ in range(3):
        formations = []
        for _ in range(3):
            positions = list(rec[pos : pos + 11])
            pos += 11
            yx = [(rec[pos + 2 * k], rec[pos + 2 * k + 1]) for k in range(11)]
            pos += 22
            formations.append((positions, yx))
        style = list(rec[pos : pos + 7])  # attacking_style, buildup, attacking_zone, positioning, defensive_style, containment_area, pressure
        pos += 7 + 2
        instructions = []
        for _ in range(4):
            instructions.append((rec[pos], rec[pos + 4]))
            pos += 8
        sliders = list(rec[pos : pos + 5])  # support_range, numbers_in_attack, defensive_line, compactness, numbers_in_defense
        pos += 5 + 0xB
        fluid = rec[pos]
        pos += 1 + 3
        presets.append((formations, style, instructions, sliders, fluid))
    starting = list(rec[pos : pos + 11])
    pos += 11
    bench = list(rec[pos : pos + 21])
    pos += 21 + 8
    takers = list(rec[pos : pos + 6])
    pos += 6
    join = list(rec[pos : pos + 3])
    pos += 3
    captain = rec[pos]
    pos += 1
    auto_sub, offside, preset_change = rec[pos], rec[pos + 1], rec[pos + 2]
    pos += 3 + 1
    atk_def = rec[pos]
    pos += 1 + 1 + 0x58
    assert pos == 628, pos
    return dict(presets=presets, starting=starting, bench=bench, takers=takers, join=join, captain=captain,
                auto=(auto_sub, offside, preset_change, atk_def))


def save_tactical_data(t):
    """`save_tactical_data`'s write order: the 405-byte block."""
    out = bytearray()
    for formations, style, instructions, sliders, fluid in t["presets"]:
        for positions, yx in formations:
            out += bytes(positions)
            for y, x in yx:
                out += bytes((y, x))
        out += bytes(style)
        for instr, player in instructions:
            out += bytes((translate_adv_instruction(instr, 19), player))
        out += bytes(sliders)
        out.append(fluid)
    out += bytes(t["starting"]) + bytes(t["bench"]) + bytes(t["takers"]) + bytes(t["join"]) + bytes(t["auto"])
    assert len(out) == 405, len(out)
    return bytes(out)


def nightly():
    payload = zlib.decompress(PES19_PAYLOAD.read_bytes())
    count = struct.unpack_from("<H", payload, 0x64)[0]
    index = next(i for i in range(count) if struct.unpack_from("<I", payload, 0x5BCC7C + i * 416)[0] == CCT_TEAM)
    rec = payload[0x69EC8C + index * 628 : 0x69EC8C + (index + 1) * 628]
    assert struct.unpack_from("<I", rec, 0)[0] == CCT_TEAM
    t = read_tactics19(rec)
    block = save_tactical_data(t)
    header = b"001" + b"19" + str(CCT_TEAM).encode().ljust(8, b"\0")
    data = header + block
    write(OUT / "pes19_tactics.4cct", data)
    print(f"pes19_tactics.4cct: {len(data)} B (synthesized); team {CCT_TEAM} is team record {index}")
    print(f"  starting {t['starting']} bench {t['bench']} takers {t['takers']} join {t['join']} captain {t['captain']} auto {t['auto']}")
    for i, (_, style, instructions, sliders, fluid) in enumerate(t["presets"]):
        print(f"  preset {i}: style {style} instructions(raw) {instructions} sliders {sliders} fluid {fluid}")
    print(f"  block md5 = {hashlib.md5(block).hexdigest()}")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    for v in (18, 19, 21):
        ted(v)
    texport17()
    squad()
    nightly()
