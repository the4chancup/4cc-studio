"""Extract the pes_savefile container fixtures from the real saves.

Per version: `pesNN_head.bin` (the file prefix up to and including the encrypted description:
salt + header + description on 16-21; seed + digests + description + logo length on 15),
`pesNN_payload_enc_head.bin` (the first 4096 encrypted payload bytes) and `pesNN_payload.bin.zz`
(the whole decrypted payload, zlib level 9). Prints the literals the tests assert.
"""
import hashlib
import struct
import sys
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from save_census import KEYS, crypt_stream, effective, file_key, lcg, section_key  # noqa: E402

OUT = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\pes_savefile\tests\fixtures")
PREFIX = 4096

SOURCES = {
    15: r"C:\Data\4cc\Tools_4cc\Midcupping\EDIT.bin",
    16: r"C:\Data\Documents\KONAMI\Pro Evolution Soccer 2016\save\EDIT00000000",
    17: r"C:\Data\Documents\KONAMI\Pro Evolution Soccer 2017\save\EDIT00000000",
    18: r"C:\Data\Documents\KONAMI\PRO EVOLUTION SOCCER 2018\save\EDIT00000000",
    19: r"C:\Data\Documents\KONAMI\PRO EVOLUTION SOCCER 2019\292733975847239680\save\EDIT00000000",
    20: r"C:\Data\Documents\KONAMI\eFootball PES 2020\292733975847239680\save\EDIT00000000-day0",
    21: r"C:\Data\Documents\KONAMI\eFootball PES 2021 SEASON UPDATE\292733975847239680\save\EDIT00000000",
}
HEADER_SIZE = {16: 176, 17: 176, 18: 208, 19: 208, 20: 208, 21: 208}


def write(path, data):
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_bytes(data)
    tmp.replace(path)


def keyed(version, data):
    key = KEYS[version]
    eff = effective(key)
    salt = data[0:320]
    fkey = file_key(salt, eff)
    hsize = HEADER_SIZE[version]
    header = crypt_stream(section_key(fkey, hsize), data[320 : 320 + hsize])
    assert header[0:64] == eff, version
    payload_size, logo_size, desc_size, serial_size = struct.unpack_from("<4I", header, 64)
    off = 320 + hsize
    desc = crypt_stream(section_key(fkey, 0), data[off : off + desc_size])
    head_end = off + desc_size
    off = head_end + logo_size
    payload_enc = data[off : off + payload_size]
    payload = crypt_stream(section_key(fkey, 2), payload_enc)
    off += payload_size
    serial = crypt_stream(section_key(fkey, 3), data[off : off + serial_size * 2])
    assert off + serial_size * 2 == len(data)
    assert desc[:9] == b"Edit Data" and desc[9:] == bytes(len(desc) - 9), desc
    # the payload prefix decrypts on its own (stream cipher): check the claim the test relies on
    assert crypt_stream(section_key(fkey, 2), payload_enc[:PREFIX]) == payload[:PREFIX]
    write(OUT / f"pes{version}_head.bin", data[:head_end])
    write(OUT / f"pes{version}_payload_enc_head.bin", payload_enc[:PREFIX])
    write(OUT / f"pes{version}_payload.bin.zz", zlib.compress(payload, 9))
    print(f"pes{version}: head {head_end} B; header {hsize}; payload {payload_size} logo {logo_size} desc {desc_size} serial {serial_size}")
    print(f"  identifier (header[80..]) = {header[80:].hex()}")
    print(f"  serial text = {serial.decode('utf-16-le')!r}")
    print(f"  payload md5 = {hashlib.md5(payload).hexdigest()}  zz {len(zlib.compress(payload, 9))} B")


def pes15(version, data):
    seed = data[0]
    desc = lcg(seed, data[49 : 49 + 384])
    off = 49 + 384
    (logo_len,) = struct.unpack_from("<I", data, off)
    head_end = off + 4
    off = head_end + logo_len
    (payload_len,) = struct.unpack_from("<I", data, off)
    off += 4
    payload_enc = data[off : off + payload_len]
    payload = lcg(seed, payload_enc)
    assert off + payload_len == len(data)
    assert hashlib.md5(desc).digest() == data[1:17]
    assert hashlib.md5(payload).digest() == data[33:49]
    assert desc[:9] == b"Edit Data" and desc[9:] == bytes(len(desc) - 9), desc
    assert lcg(seed, payload_enc[:PREFIX]) == payload[:PREFIX]
    write(OUT / f"pes{version}_head.bin", data[:head_end])
    write(OUT / f"pes{version}_payload_enc_head.bin", payload_enc[:PREFIX])
    write(OUT / f"pes{version}_payload.bin.zz", zlib.compress(payload, 9))
    print(f"pes15: head {head_end} B; seed {seed}; payload {payload_len} logo {logo_len}")
    print(f"  digests desc {data[1:17].hex()} logo {data[17:33].hex()} payload {data[33:49].hex()}")
    print(f"  payload md5 = {hashlib.md5(payload).hexdigest()}  zz {len(zlib.compress(payload, 9))} B")


OUT.mkdir(parents=True, exist_ok=True)
# Versions to extract as arguments (`save_fixtures.py 20`); none means every source.
versions = [int(a) for a in sys.argv[1:]] or list(SOURCES)
for version in versions:
    data = Path(SOURCES[version]).read_bytes()
    if version == 15:
        pes15(version, data)
    else:
        keyed(version, data)
