"""ftex converge fixtures (2.20d), each checked against pes-file-tools' reference reader.

- handbuilt_raw_frames.ftex: konami_bc1_bibs_metalness.dds with every frame stored raw
  (chunk count 0, compressed size 0), the storage Konami uses in ggx_lookup_rgba.ftex and
  montage/mark.ftex (13 raw frames among the 10370 PES 2021 files).
- handbuilt_single_zlib.ftex: the same DDS with every frame one zlib stream (chunk count 0,
  compressed size > 0), a storage the reference reads and no PES 2021 file uses.
- pft_volume_rgba16f.ftex / .dds: a 4x4x4 R16G16B16A16_FLOAT volume DDS with three mips run
  through pes-file-tools' ddsToFtexBuffer, and that FTEX converted back by ftexToDdsBuffer
  (the reference DDS). No volume texture exists among the small Konami files (lut_Match.ftex,
  33x33x33, is the only one, 181 KB).
"""
import hashlib
import struct
import sys
import zlib
from pathlib import Path

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\pes-file-tools\lib")
from pes_file_tools.ftex import ddsToFtexBuffer, ftexToDdsBuffer  # noqa: E402

DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\ftex\tests\fixtures")
MIP = struct.Struct("< I I I BB H")


def put(name, data):
    (DEST / name).write_bytes(data)
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


def mip_frames(dds, sizes):
    frames, at = [], 128
    for size in sizes:
        frames.append(dds[at:at + size])
        at += size
    assert at == len(dds), (at, len(dds))
    return frames


def rebuild(header64, frames, compress):
    """FTEX with chunk-count-0 frames: raw when compress is False, one zlib stream otherwise."""
    records, body = [], b""
    base = 64 + 16 * len(frames)
    for index, frame in enumerate(frames):
        stored = zlib.compress(frame, 9) if compress else frame
        records.append(MIP.pack(base + len(body), len(frame), len(stored) if compress else 0, index, 0, 0))
        body += stored
    return header64 + b"".join(records) + body


bc1_dds = (DEST / "konami_bc1_bibs_metalness.dds").read_bytes()
header = bytes(ddsToFtexBuffer(bc1_dds, "LINEAR"))[:64]
frames = mip_frames(bc1_dds, [128, 32, 8])  # 16x16, 8x8, 4x4 BC1
for name, compress in [("handbuilt_raw_frames.ftex", False), ("handbuilt_single_zlib.ftex", True)]:
    ftex = rebuild(header, frames, compress)
    assert bytes(ftexToDdsBuffer(ftex)) == bc1_dds, name
    put(name, ftex)

# 4x4x4 volume, R16G16B16A16_FLOAT (8 bytes per texel), mips 4x4x4, 2x2x2, 1x1x1.
dds_header = struct.pack(
    "< 4s 7I 44x 2I 4s 5I 2I 12x",
    b"DDS ", 124, 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000 | 0x800000, 4, 4, 4 * 8, 4, 3,
    32, 0x4, b"DX10", 0, 0, 0, 0, 0,
    0x1000 | 0x8 | 0x400000, 0x200000,
)
dx10 = struct.pack("< 5I", 10, 4, 0, 1, 0)
texels = bytes((i * 37 + 11) % 256 for i in range(512 + 64 + 8))
volume_dds = dds_header + dx10 + texels
volume_ftex = bytes(ddsToFtexBuffer(volume_dds, "LINEAR"))
reference_dds = bytes(ftexToDdsBuffer(volume_ftex))
assert reference_dds[148:] == texels
put("pft_volume_rgba16f.ftex", volume_ftex)
put("pft_volume_rgba16f.dds", reference_dds)
