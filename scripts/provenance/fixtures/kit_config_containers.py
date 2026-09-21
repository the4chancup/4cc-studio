"""Build the kit_config version fixtures: Blue's bundled `UniformParameter18.bin` and
`UniformParameter19.bin` (the 4cc kit configs of those seasons, written by the Kit Manager
variants and by Red/Blue), WESYS-wrapped the way the game stores the container so the fixtures
stay small. `UniformParameter::read` unwraps them. New files only."""
import struct
import zlib
from pathlib import Path

SRC = Path(r"C:\Data\4cc\4cc aet compiler\4cc-aet-compiler-blue\lib\bins")
DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\kit_config\tests\fixtures")
NAMES = {"UniformParameter18.bin": "blue_pes18_UniformParameter.bin", "UniformParameter19.bin": "blue_pes19_UniformParameter.bin"}

for src_name, dest_name in NAMES.items():
    out = DEST / dest_name
    assert not out.exists(), out
    raw = (SRC / src_name).read_bytes()
    assert raw[3:8] != b"WESYS", "already wrapped"
    count, table = struct.unpack_from("<II", raw, 0)
    body = zlib.compress(raw, 9)
    wrapped = bytes([0x00, 0x10, 0x01]) + b"WESYS" + struct.pack("<II", len(body), len(raw)) + body
    out.write_bytes(wrapped)
    print(f"{dest_name}: {count} entries, {len(raw)} -> {len(wrapped)} bytes")
