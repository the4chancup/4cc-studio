"""Container census over every PES save on this machine.

For each file: try PES 15's keyless format, then every 16-21 master key with both header
sizes (176/208); report the version that passes the integrity compare, the section sizes,
the identifier bytes, the serial text, the plan's player-count and anchor checks, and how
well the decrypted payload compresses (fixture sizing).
"""
import hashlib
import struct
import sys
import zlib
from pathlib import Path

KEYS = {
    16: bytes([
        0x4D, 0x55, 0x94, 0x66, 0xD9, 0x62, 0x5C, 0xEC, 0xC1, 0x7C, 0x48, 0x36, 0x77, 0x31, 0x50, 0xE1,
        0x87, 0x1C, 0xB5, 0x6B, 0x41, 0xD4, 0x92, 0x4F, 0x4A, 0x8C, 0x71, 0x27, 0x0A, 0x0D, 0x50, 0x63,
        0x94, 0x2B, 0x58, 0x5E, 0x99, 0x0B, 0x8B, 0x97, 0x96, 0x66, 0xC0, 0x00, 0xB7, 0x1D, 0x72, 0x75,
        0xD6, 0xE8, 0x5B, 0x0E, 0xAF, 0xF1, 0x72, 0xD1, 0xB1, 0xE3, 0x3C, 0x75, 0xDE, 0x9C, 0x13, 0x09]),
    "16mc": bytes([
        0x85, 0x91, 0x1E, 0x0F, 0x32, 0xE4, 0xBA, 0xD7, 0x45, 0x2C, 0xBC, 0xF1, 0x06, 0x16, 0x85, 0x94,
        0xCE, 0x43, 0x52, 0xF5, 0x41, 0x54, 0x87, 0x8B, 0xCB, 0x88, 0x46, 0xC6, 0x15, 0xB7, 0xB3, 0xDC,
        0x36, 0x1F, 0x13, 0x62, 0xF9, 0x03, 0xD4, 0x00, 0xAD, 0xD2, 0xE3, 0xC3, 0x91, 0xE6, 0x2F, 0x31,
        0x79, 0x66, 0xB5, 0x5A, 0x00, 0x9F, 0x93, 0x56, 0xB0, 0xB7, 0x43, 0x7C, 0x2F, 0x4C, 0x97, 0xAB]),
    17: bytes([
        0x9B, 0xC7, 0x13, 0x28, 0x2D, 0xE8, 0x47, 0x75, 0x4D, 0x52, 0x9E, 0x35, 0x90, 0xAA, 0x6A, 0x7A,
        0x5C, 0xFA, 0x60, 0x9F, 0x6A, 0x32, 0x04, 0x57, 0xB8, 0x9F, 0x59, 0xA5, 0x5F, 0xAC, 0x7D, 0x62,
        0xFE, 0x10, 0x2A, 0xD6, 0x95, 0xFA, 0xDF, 0xA0, 0x68, 0xBD, 0x40, 0x95, 0x47, 0x9C, 0xBB, 0x40,
        0xF2, 0x94, 0x49, 0x3C, 0xC8, 0xE0, 0x94, 0x9D, 0x7B, 0x01, 0x6F, 0xF2, 0xC5, 0x3A, 0x2C, 0xE5]),
    18: bytes([
        0x47, 0x51, 0x83, 0x38, 0x83, 0x9E, 0x85, 0x3A, 0xFA, 0x39, 0x08, 0xDC, 0x86, 0xAF, 0x0C, 0xB5,
        0xC4, 0x0E, 0xE4, 0xE4, 0x15, 0xE3, 0xCB, 0x64, 0x1B, 0x8A, 0xF1, 0xE5, 0x2C, 0xBD, 0x4D, 0x1B,
        0xA4, 0xD6, 0xA1, 0x97, 0x2C, 0x4B, 0xD1, 0xDA, 0xFA, 0xCD, 0x7E, 0xB4, 0x8A, 0x18, 0xC9, 0xE8,
        0xC5, 0x3E, 0x1A, 0x8B, 0x19, 0x45, 0x01, 0xF8, 0x8B, 0x85, 0x97, 0x8B, 0x78, 0x7B, 0x3C, 0x2E]),
    19: bytes([
        0xFD, 0x60, 0x4A, 0x3E, 0xFD, 0x69, 0x20, 0xD1, 0x93, 0x92, 0x37, 0xD7, 0x60, 0xD8, 0x30, 0xEE,
        0x65, 0x66, 0xFD, 0x6C, 0xE6, 0x9E, 0x48, 0xF8, 0x0A, 0x0D, 0xC1, 0x23, 0x7F, 0xAC, 0x89, 0x05,
        0x1D, 0xF8, 0x5A, 0x79, 0x10, 0x7E, 0xAD, 0x81, 0xAC, 0xAE, 0x9A, 0x6A, 0xAB, 0x16, 0xA6, 0x81,
        0xC2, 0xD2, 0x18, 0xC0, 0xF4, 0xE6, 0x5C, 0x27, 0x74, 0xF6, 0xC1, 0x9F, 0xF5, 0x01, 0x38, 0x72]),
    20: bytes([
        0xE2, 0xBF, 0x51, 0x07, 0x54, 0xE6, 0x21, 0x78, 0x2C, 0x5E, 0x8D, 0x33, 0x13, 0x7A, 0xC9, 0x15,
        0x99, 0x77, 0xD9, 0xA0, 0x1B, 0xC2, 0x95, 0xD9, 0xBB, 0x9B, 0xB1, 0x00, 0x84, 0x1C, 0xB3, 0x62,
        0xE5, 0x40, 0xD9, 0x56, 0x45, 0x5B, 0x7C, 0x7C, 0x4F, 0xF1, 0xDA, 0x26, 0xB4, 0x5A, 0x0C, 0x5C,
        0x4D, 0x6B, 0x9E, 0x98, 0x75, 0xA9, 0x39, 0x07, 0x4C, 0x4B, 0x55, 0xBD, 0x8E, 0x01, 0xA9, 0x31]),
    21: bytes([
        0x90, 0x61, 0xD8, 0x66, 0x43, 0x77, 0x24, 0xF8, 0x92, 0xBA, 0xB8, 0x71, 0x21, 0xC7, 0x60, 0x63,
        0xF0, 0x91, 0x9A, 0x7D, 0xED, 0x47, 0x80, 0xDE, 0x51, 0xF5, 0xDD, 0xD1, 0x08, 0xFE, 0x32, 0x84,
        0xF5, 0x09, 0x92, 0x00, 0xB2, 0x3E, 0x88, 0x9F, 0xEB, 0x24, 0x43, 0x05, 0x58, 0x76, 0x00, 0x22,
        0x9B, 0xFE, 0xEC, 0xF6, 0x50, 0x00, 0x29, 0xD3, 0x42, 0x75, 0x50, 0xB9, 0xEC, 0xD2, 0xF6, 0x75]),
}

# Plan table: version -> (count offset, player block offset, record size, appearance offset or
# in-record offset, team ids, rosters, tactics)
PLAN = {
    15: (0x34, 0x4C, 112, ("array", 0x2AB9CC, 68), 0x44AA6C, 0x4E45CC, 0x507194),
    16: (0x34, 0x4C, 112, ("array", 0x2AB9CC, 72), 0x46310C, 0x4FCC6C, 0x51F814),
    17: (0x5C, 0x78, 188, ("record", 116), 0x3C3E58, 0x475A90, 0x490640),
    18: (0x60, 0x7C, 188, ("record", 116), 0x3C3E5C, 0x46FF54, 0x488B74),
    19: (0x60, 0x7C, 188, ("record", 116), 0x5BCC7C, 0x6773C4, 0x69EC8C),
    20: (0x60, 0x7C, 312, ("record", 240), 0x8ED2FC, 0x9CCC04, 0xA01E3C),
    21: (0x60, 0x7C, 312, ("record", 240), 0x8ED2FC, 0x9D4648, 0xA09880),
}


class MT:
    def __init__(self, key_words):
        self.mt = [0] * 624
        self.mt[0] = 19650218
        for i in range(1, 624):
            self.mt[i] = (1812433253 * (self.mt[i - 1] ^ (self.mt[i - 1] >> 30)) + i) & 0xFFFFFFFF
        i, j = 1, 0
        for _ in range(max(624, len(key_words))):
            self.mt[i] = ((self.mt[i] ^ ((self.mt[i - 1] ^ (self.mt[i - 1] >> 30)) * 1664525)) + key_words[j] + j) & 0xFFFFFFFF
            i += 1
            j += 1
            if i >= 624:
                self.mt[0] = self.mt[623]
                i = 1
            if j >= len(key_words):
                j = 0
        for _ in range(623):
            self.mt[i] = ((self.mt[i] ^ ((self.mt[i - 1] ^ (self.mt[i - 1] >> 30)) * 1566083941)) - i) & 0xFFFFFFFF
            i += 1
            if i >= 624:
                self.mt[0] = self.mt[623]
                i = 1
        self.mt[0] = 0x80000000
        self.index = 624

    def next(self):
        if self.index >= 624:
            for i in range(624):
                y = (self.mt[i] & 0x80000000) | (self.mt[(i + 1) % 624] & 0x7FFFFFFF)
                v = self.mt[(i + 397) % 624] ^ (y >> 1)
                if y & 1:
                    v ^= 0x9908B0DF
                self.mt[i] = v
            self.index = 0
        y = self.mt[self.index]
        self.index += 1
        y ^= y >> 11
        y ^= (y << 7) & 0x9D2C5680
        y ^= (y << 15) & 0xEFC60000
        y ^= y >> 18
        return y & 0xFFFFFFFF


def rol(v, n):
    return ((v << n) & 0xFFFFFFFF) | (v >> (32 - n))


def ror(v, n):
    return rol(v, 32 - n)


def crypt_stream(key, data):
    mt = MT(list(struct.unpack("<16I", key)))
    c0, c1, c2, c3 = mt.next(), mt.next(), mt.next(), mt.next()
    words = (len(data) + 3) // 4
    out = bytearray(words * 4)
    padded = bytes(data) + bytes(words * 4 - len(data))
    for i in range(words):
        c4 = mt.next()
        v = c4 ^ c3 ^ c2 ^ c1 ^ c0
        struct.pack_into("<I", out, i * 4, v ^ struct.unpack_from("<I", padded, i * 4)[0])
        c0, c1, c2, c3 = ror(c1, 15), rol(c2, 11), rol(c3, 7), ror(c4, 13)
    return bytes(out[: len(data)])


def xor(a, b):
    return bytes(x ^ y for x, y in zip(a, b))


def effective(key):
    return bytes(key[(i & ~7) + 7 - (i & 7)] for i in range(64))


def file_key(salt, eff):
    header_key = xor(eff, salt[256:320])
    dec = crypt_stream(header_key, salt[0:256]) + salt[256:320]
    k = dec[0:64]
    for b in range(1, 5):
        k = xor(k, dec[b * 64 : (b + 1) * 64])
    return k


def section_key(fkey, n):
    return xor(fkey, struct.pack("<Q", n) * 8)


def try_16_21(data):
    salt = data[0:320]
    for name, key in KEYS.items():
        eff = effective(key)
        fkey = file_key(salt, eff)
        for header_size in (176, 208):
            header = crypt_stream(section_key(fkey, header_size), data[320 : 320 + header_size])
            if header[0:64] == eff:
                return name, header_size, fkey, header
    return None


def lcg(seed, data):
    out = bytearray(len(data))
    c = seed
    for i, b in enumerate(data):
        c = (c * 21 + 7) % 32768
        out[i] = b ^ (c % 255)
    return bytes(out)


def try_15(data):
    seed = data[0]
    desc = lcg(seed, data[49 : 49 + 384])
    off = 49 + 384
    (logo_len,) = struct.unpack_from("<I", data, off)
    off += 4
    logo = lcg(seed, data[off : off + logo_len])
    off += logo_len
    (payload_len,) = struct.unpack_from("<I", data, off)
    off += 4
    payload = lcg(seed, data[off : off + payload_len])
    off += payload_len
    ok = (
        hashlib.md5(desc).digest() == data[1:17]
        and hashlib.md5(logo).digest() == data[17:33]
        and hashlib.md5(payload).digest() == data[33:49]
    )
    return ok, desc, logo, payload, off == len(data)


def printable(b):
    return "".join(chr(c) if 32 <= c < 127 else "." for c in b)


def anchors(payload, version):
    count_off, block_off, rec, appearance, team_ids, rosters, tactics = PLAN[version]
    (count,) = struct.unpack_from("<H", payload, count_off)
    (count36,) = struct.unpack_from("<H", payload, 0x36) if version in (15, 16) else (None,)
    result = [f"count@{count_off:#x}={count}"]
    if count36 is not None:
        result.append(f"count@0x36={count36}")
    ids_ok = 0
    ids_total = min(count, 20000)
    if appearance[0] == "record":
        for i in range(ids_total):
            base = block_off + rec * i
            (pid,) = struct.unpack_from("<I", payload, base)
            (aid,) = struct.unpack_from("<I", payload, base + appearance[1])
            ids_ok += pid == aid
        result.append(f"appearance id == player id: {ids_ok}/{ids_total}")
    else:
        _, arr, size = appearance
        first_ids = {struct.unpack_from("<I", payload, block_off + rec * i)[0] for i in range(ids_total)}
        app_ids = {struct.unpack_from("<I", payload, arr + size * i)[0] for i in range(ids_total)}
        result.append(f"appearance ids == player ids as sets: {first_ids == app_ids}; shared {len(first_ids & app_ids)}")
    (last_id,) = struct.unpack_from("<I", payload, block_off + rec * (count - 1))
    result.append(f"last player id {last_id}")
    # team id section: first few u32
    tids = struct.unpack_from("<8I", payload, team_ids)
    result.append(f"team ids @{team_ids:#x}: {tids[:8]}")
    return "; ".join(result)


def main():
    files = [Path(p) for p in sys.argv[1:]]
    for path in files:
        data = path.read_bytes()
        print(f"\n== {path} ({len(data)} bytes)")
        r = try_16_21(data) if len(data) > 600 else None
        if r:
            name, header_size, fkey, header = r
            payload_size, logo_size, desc_size, serial_size = struct.unpack_from("<4I", header, 64)
            ident = header[80:]
            off = 320 + header_size
            desc = crypt_stream(section_key(fkey, 0), data[off : off + desc_size]); off += desc_size
            logo = crypt_stream(section_key(fkey, 1), data[off : off + logo_size]); off += logo_size
            payload = crypt_stream(section_key(fkey, 2), data[off : off + payload_size]); off += payload_size
            serial = crypt_stream(section_key(fkey, 3), data[off : off + serial_size * 2]); off += serial_size * 2
            print(f"  key {name}, header {header_size}; payload {payload_size}, logo {logo_size}, desc {desc_size}, serial {serial_size}*2; consumed {off}/{len(data)}")
            print(f"  identifier[0:64] hex {header[80:144].hex()}")
            print(f"  identifier text '{printable(ident)}'")
            print(f"  serial '{printable(serial)}' logo starts {logo[:8].hex()}")
            print(f"  desc '{printable(desc[:64])}'")
            version = {"16mc": 16}.get(name, name)
            print("  anchors:", anchors(payload, version))
            print(f"  payload zlib-9 size {len(zlib.compress(payload, 9))}")
        else:
            ok, desc, logo, payload, exact = try_15(data)
            print(f"  PES15 keyless: md5 ok {ok}; exact length {exact}; payload {len(payload)}, logo {len(logo)}")
            if ok:
                print(f"  desc '{printable(desc[:64])}'")
                print("  anchors:", anchors(payload, 15))
                print(f"  payload zlib-9 size {len(zlib.compress(payload, 9))}")


main()
