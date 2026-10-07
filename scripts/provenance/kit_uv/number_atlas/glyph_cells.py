"""Per-digit-cell ink bounding boxes of stock kit number atlases, PES 17 against PES 21.

For every kit stem whose `_back`, `_chest` and `_leg` exist in both games (or the first N
of them), decodes each atlas's top mip, splits it into ten cells along its long axis
(cell i = [i*L/10, (i+1)*L/10), rounded to whole pixels by floor), and records per cell the
ink bounding box as fractions of the cell (x0, x1, y0, y1; x across the texture, y down it)
plus the ink's size in pixels. Ink = alpha > 32 when the texture has alpha below 200
somewhere, else luma sum > 96 (the rule `kit_leg_atlas.py` uses).

Writes glyph_cells.csv (one row per stem, suffix, game, cell) next to this script.
Usage: python glyph_cells.py [max stems, default all]"""
import csv
import glob
import io
import os
import sys
import tempfile
import zlib

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib")
import Ftex  # noqa: E402
import numpy as np  # noqa: E402
from PIL import Image  # noqa: E402

T17 = r"E:\PES2017\Data\dt34_win_files\common\character0\model\character\uniform\texture"
T21 = r"E:\PES2021\Data\dt34_g4_files\Asset\model\character\uniform\texture\#windx11"
HERE = os.path.dirname(os.path.abspath(__file__))
SUFFIXES = ("back", "chest", "leg")


def load_dds(path):
    data = open(path, "rb").read()
    if data[3:8] == b"WESYS":
        data = zlib.decompress(data[16:])
    return np.asarray(Image.open(io.BytesIO(data)).convert("RGBA")).astype(np.int32)


def load_ftex(path, tmp):
    dds = os.path.join(tmp, "x.dds")
    Ftex.ftexToDds(path, dds)
    return load_dds(dds)


def ink_mask(rgba):
    alpha = rgba[..., 3]
    if alpha.min() < 200:
        return alpha > 32, "alpha"
    return rgba[..., :3].sum(-1) > 96, "luma"


def cells(rgba):
    """Yields (index, cell ink mask) along the long axis."""
    mask, _ = ink_mask(rgba)
    h, w = mask.shape
    column = h > w
    length = h if column else w
    for i in range(10):
        a, b = i * length // 10, (i + 1) * length // 10
        yield i, (mask[a:b, :] if column else mask[:, a:b])


def bbox(cell):
    ys, xs = np.nonzero(cell)
    h, w = cell.shape
    if len(xs) == 0:
        return None
    x0, x1, y0, y1 = xs.min(), xs.max() + 1, ys.min(), ys.max() + 1
    return (x0 / w, x1 / w, y0 / h, y1 / h, x1 - x0, y1 - y0, w, h, int(cell.sum()))


def main():
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else None
    stems17 = {os.path.basename(p)[: -len("_back.dds")] for p in glob.glob(os.path.join(T17, "*_back.dds"))}
    stems21 = {os.path.basename(p)[: -len("_back.ftex")] for p in glob.glob(os.path.join(T21, "*_back.ftex"))}
    common = sorted(stems17 & stems21)
    print(f"stems: PES17 {len(stems17)}, PES21 {len(stems21)}, common {len(common)}")
    if limit:
        common = common[:limit]
    out = open(os.path.join(HERE, "glyph_cells.csv"), "w", newline="")
    writer = csv.writer(out)
    writer.writerow(["stem", "suffix", "game", "tex_w", "tex_h", "ink_rule", "cell",
                     "x0", "x1", "y0", "y1", "ink_w_px", "ink_h_px", "cell_w_px", "cell_h_px", "ink_px"])
    with tempfile.TemporaryDirectory() as tmp:
        for n, stem in enumerate(common):
            for suffix in SUFFIXES:
                p17 = os.path.join(T17, f"{stem}_{suffix}.dds")
                p21 = os.path.join(T21, f"{stem}_{suffix}.ftex")
                if not (os.path.exists(p17) and os.path.exists(p21)):
                    continue
                for game, rgba in (("17", load_dds(p17)), ("21", load_ftex(p21, tmp))):
                    _, rule = ink_mask(rgba)
                    h, w = rgba.shape[:2]
                    for i, cell in cells(rgba):
                        box = bbox(cell)
                        row = [stem, suffix, game, w, h, rule, i]
                        row += ["" for _ in range(9)] if box is None else [
                            f"{box[0]:.4f}", f"{box[1]:.4f}", f"{box[2]:.4f}", f"{box[3]:.4f}", *box[4:]]
                        writer.writerow(row)
            if n % 50 == 0:
                print(f"{n} {stem}", flush=True)
    out.close()


if __name__ == "__main__":
    main()
