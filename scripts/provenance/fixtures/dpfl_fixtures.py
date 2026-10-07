"""Writes team_compiler's tests/fixtures/dpfl/ (step 4.21): the three-entry list the working-bin
walk's tests install, and copies of two real lists whose bytes outside the names are not zero.

The three-entry list is built in the layout measured on the maintainer's PES 2017 and PES 2021
lists (docs/plans/team_compiler/pipeline.md "DpFileList upgrade"): a 16-byte header (u32 0,
u32 entry count, 8 zero bytes), 48-byte records holding the NUL-padded CPK file name, then the
PES 2021 list's 1204-byte zero tail.
"""
import os
import struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "crates/tools/team_compiler/tests/fixtures/dpfl"

REAL = {
    "pes19_official.bin": Path("C:/Program Files/PES2019/download/DpFileList.bin"),
    "pes20_header_100.bin": Path("S:/Games/PES2020/download/DpFileList.bin"),
}
WALK_ENTRIES = ["4cc_08_bins.cpk", "4cc_61_midcup.cpk", "4cc_99_test.cpk"]
PES21_TAIL = 1204


def record(name: str) -> bytes:
    raw = name.encode("ascii")
    assert len(raw) < 48
    return raw + bytes(48 - len(raw))


def names(data: bytes) -> list[str]:
    count = struct.unpack_from("<I", data, 4)[0]
    out = []
    for i in range(count):
        rec = data[16 + i * 48:16 + (i + 1) * 48]
        out.append(rec[:rec.index(0)].decode("ascii"))
    assert not any(data[16 + count * 48:]), "the tail after the records is zero"
    return out


def write(path: Path, data: bytes) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_bytes(data)
    os.replace(temp, path)


def main() -> None:
    walk = struct.pack("<4I", 0, len(WALK_ENTRIES), 0, 0)
    walk += b"".join(record(name) for name in WALK_ENTRIES) + bytes(PES21_TAIL)
    assert names(walk) == WALK_ENTRIES
    assert len(walk) == 16 + 3 * 48 + PES21_TAIL

    real = {}
    for name, source in REAL.items():
        data = source.read_bytes()
        listed = names(data)
        assert listed[0] == "4cc_01_db.cpk", listed[0]
        real[name] = data
    pes19 = real["pes19_official.bin"]
    assert struct.unpack_from("<I", pes19, 4)[0] == 29
    assert pes19[16 + 36] == 28, "PES 19's first record carries 28 at offset 36"
    assert names(pes19)[-1] == "4cc_69_midcup.cpk"
    pes20 = real["pes20_header_100.bin"]
    assert struct.unpack_from("<2I", pes20, 0) == (100, 45)
    assert names(pes20)[-1] == "4cc_90_test.cpk"

    OUT.mkdir(parents=True, exist_ok=True)
    write(OUT / "walk.bin", walk)
    for name, data in real.items():
        write(OUT / name, data)
    for path in sorted(OUT.glob("*.bin")):
        print(path.name, path.stat().st_size)


if __name__ == "__main__":
    main()
