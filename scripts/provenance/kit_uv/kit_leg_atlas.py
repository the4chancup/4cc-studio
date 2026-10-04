"""Whether the shorts-number texture (`_leg`) is laid out the same in both engines.

`_leg` is a glyph atlas the game reads cell by cell, not a texture mapped through the uniform
models' UVs, so the kit layout change cannot reach it unless the atlas's own grid changed. This
compares one stock team's `_leg` from each game: size, and which columns and rows of the atlas
hold any ink (alpha, or luma when the texture has no alpha), printed as runs in fractions of
the texture's size so two resolutions compare directly. Then it tallies the size of every
stock `_back`, `_chest`, `_leg` and `_name` texture of each game, read from the headers.
Usage: python kit_leg_atlas.py [team kit stem, default a0150p1]"""
import collections
import glob
import io
import os
import struct
import sys
import tempfile
import zlib

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib")
import Ftex
import numpy as np
from PIL import Image

T17 = r"E:\PES2017\Data\dt34_win_files\common\character0\model\character\uniform\texture"
T21 = r"E:\PES2021\Data\dt34_g4_files\Asset\model\character\uniform\texture\#windx11"


def runs(mask):
    """Runs of True as (start, end) fractions of the axis, three decimals."""
    out, start = [], None
    for i, on in enumerate(list(mask) + [False]):
        if on and start is None:
            start = i
        elif not on and start is not None:
            out.append(f"{start / len(mask):.3f}-{i / len(mask):.3f}")
            start = None
    return out


def describe(name, path):
    data = open(path, "rb").read()
    if data[3:8] == b"WESYS":
        # The games' own files come zlib-wrapped: a 16-byte header, then the stream.
        data = zlib.decompress(data[16:])
    im = Image.open(io.BytesIO(data))
    rgba = np.asarray(im.convert("RGBA")).astype(np.int32)
    alpha = rgba[..., 3]
    ink = alpha > 32 if alpha.min() < 200 else rgba[..., :3].sum(-1) > 96
    print(f"== {name}: {os.path.basename(path)} {im.size[0]}x{im.size[1]} {im.mode}; ink share {ink.mean():.3f}")
    print("   columns with ink: " + " ".join(runs(ink.any(0))))
    print("   rows with ink:    " + " ".join(runs(ink.any(1))))


stem = sys.argv[1] if len(sys.argv) > 1 else "a0150p1"
describe("pre-Fox", os.path.join(T17, f"{stem}_leg.dds"))
with tempfile.TemporaryDirectory() as tmp:
    dds = os.path.join(tmp, "leg.dds")
    Ftex.ftexToDds(os.path.join(T21, f"{stem}_leg.ftex"), dds)
    describe("Fox", dds)


def dds_size(path):
    data = open(path, "rb").read()
    if data[3:8] == b"WESYS":
        data = zlib.decompress(data[16:])
    height, width = struct.unpack_from("<II", data, 12)
    return (width, height)


def ftex_size(path):
    return struct.unpack_from("<HH", open(path, "rb").read(16), 10)


for name, folder, ext, size in (("pre-Fox", T17, ".dds", dds_size), ("Fox", T21, ".ftex", ftex_size)):
    print(f"== {name}: sizes of the stock number and name textures (width, height): count")
    for suffix in ("back", "chest", "leg", "name"):
        counts = collections.Counter(size(p) for p in glob.glob(os.path.join(folder, f"*_{suffix}{ext}")))
        print(f"   _{suffix}: " + ", ".join(f"{k}: {n}" for k, n in counts.most_common()))
