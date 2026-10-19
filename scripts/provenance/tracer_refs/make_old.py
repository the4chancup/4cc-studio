"""Cut the Fox referee export "refs lan party" down to the referee tracer's old-layout source: the
referee masterchef (the smaller of the two whose gloves/ holds models; his face/ holds
face_diff.bin, boots/ his model, common/ the textures) and the three Refs.txt slots that name
him, the large textures reduced.

Reads the copy in .tmp/6_10/source/ and writes .tmp/6_10/old/refs Tracer/ (refuses to run when
it exists).
"""

import struct
from pathlib import Path

HERE = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio-wt-g\.tmp\6_10")
SOURCE = HERE / "source" / "refs lan party"
TARGET = HERE / "old" / "refs Tracer"
REFEREE = "masterchef"

# DXT textures (one mip) reduced by keeping every n-th 4x4 block in each direction.
REDUCED = {
    "common/arm_1_bsm.dds": 8,
    "common/masterchef.dds": 8,
    "common/hat.dds": 4,
}

DDSD_LINEARSIZE = 0x80000


def reduced_dxt(data: bytes, factor: int) -> bytes:
    assert data[:4] == b"DDS ", data[:4]
    flags, height, width, _pitch, _depth, mips = struct.unpack_from("<IIIIII", data, 8)
    block = {b"DXT1": 8, b"DXT3": 16, b"DXT5": 16}[data[84:88]]
    assert mips <= 1, mips
    blocks_x, blocks_y = width // 4, height // 4
    assert len(data) == 128 + blocks_x * blocks_y * block, (len(data), width, height)
    assert blocks_x % factor == 0 and blocks_y % factor == 0
    out = bytearray()
    for by in range(0, blocks_y, factor):
        for bx in range(0, blocks_x, factor):
            start = 128 + (by * blocks_x + bx) * block
            out += data[start : start + block]
    header = bytearray(data[:128])
    struct.pack_into("<II", header, 12, height // factor, width // factor)
    assert flags & DDSD_LINEARSIZE, hex(flags)
    struct.pack_into("<I", header, 20, len(out))
    return bytes(header) + bytes(out)


assert not TARGET.exists(), f"{TARGET} exists; remove it by hand first"
referee = SOURCE / "Players" / REFEREE
assert referee.is_dir(), referee
files = sorted(path for path in referee.rglob("*") if path.is_file())
names = {path.relative_to(referee).as_posix() for path in files}
assert set(REDUCED) <= names, names

for path in files:
    relative = path.relative_to(referee).as_posix()
    data = path.read_bytes()
    if relative in REDUCED:
        data = reduced_dxt(data, REDUCED[relative])
    target = TARGET / "Players" / REFEREE / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)

# The roster keeps the referee's own lines, each with its CRLF: slots 05, 23 and 24, so one
# folder backing several slots stays in the cut.
roster = (SOURCE / "Refs.txt").read_bytes()
lines = roster.split(b"\r\n")
kept = [line for line in lines if line.split(b" ", 1)[1:] == [REFEREE.encode()]]
assert kept == [b"05 masterchef", b"23 masterchef", b"24 masterchef"], kept
(TARGET / "Refs.txt").write_bytes(b"".join(line + b"\r\n" for line in kept))

total = 0
for path in sorted(TARGET.rglob("*")):
    if path.is_file():
        total += path.stat().st_size
        print(f"{path.stat().st_size:8d}  {path.relative_to(TARGET).as_posix()}")
print("total", total)
