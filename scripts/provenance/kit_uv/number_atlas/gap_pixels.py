"""RGBA of a few stock atlases' gap and glyph pixels: what fills an atlas outside its glyphs."""
import os
import sys
import tempfile
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glyph_cells import load_dds, load_ftex  # noqa: E402
import numpy as np  # noqa: E402

T17 = r"E:\PES2017\Data\dt34_win_files\common\character0\model\character\uniform\texture"
T21 = r"E:\PES2021\Data\dt34_g4_files\Asset\model\character\uniform\texture\#windx11"
stems = ["a0150p1", "a0150p2", "a0101p1", "a0230g1"]
with tempfile.TemporaryDirectory() as tmp:
    for stem in stems:
        for kind in ("_back",):
            p21 = os.path.join(T21, stem + kind + ".ftex")
            p17 = os.path.join(T17, stem + kind + ".dds")
            for label, path, loader in (("21", p21, lambda p: load_ftex(p, tmp)), ("17", p17, load_dds)):
                if not os.path.exists(path):
                    print(stem, label, "missing"); continue
                rgba = loader(path)
                h, w = rgba.shape[:2]
                a = rgba[..., 3]
                bg = rgba[a < 8]
                ink = rgba[a > 200]
                uniq = np.unique(bg[:, :3], axis=0)
                print(f"{stem}{kind} {label} {w}x{h}: alpha min {a.min()} max {a.max()}; bg px {len(bg)}, bg rgb distinct {len(uniq)} first {uniq[:3].tolist()}; ink rgb mean {ink[:, :3].mean(0).round().tolist() if len(ink) else None}; corner {rgba[0,0].tolist()} last-col {rgba[h//2, w-1].tolist()}")
