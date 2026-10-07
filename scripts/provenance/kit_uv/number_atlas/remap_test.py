"""Which re-arrangement of a stock PES 17 column reproduces the same team's stock PES 21 row?

For every stem with `_back`/`_chest`/`_leg` in both games, re-arranges the PES 17 column into a
row the size of the PES 21 one by each candidate mapping, then compares ink masks
(glyph_cells.ink_mask) with the stock PES 21 row: IoU over the whole atlas, and per digit
the IoU inside the stock digit's own region. Work is in px of the 2048-long atlas, scaled by
f = row length / 2048 for the 1024-long ones. The 2048-long column cell is 128 x 204.8.

Candidates, each placing column cell d = rows [d*204.8, (d+1)*204.8):
  A   stretch the cell onto the equal tenth [d*204.8, (d+1)*204.8) x [0, 256);
  B   keep pixel scale (128 x 204.8), centred in the equal tenth (both axes);
  S   scale uniformly by 1.25 to 160 x 256 and put its left edge at SLOTS[d]
      (0, 190, then 200*d - 60), the layout slot_fit.py measured;
  Sf  as S with the per-digit medians slot_fit.py measured (0, 191, 344, 541, 740, 940,
      1141, 1339, 1540, 1740).
And the reverse, row -> column, compared with the stock PES 17 column:
  rS  crop [SLOTS[d], SLOTS[d] + 160) x [0, 256) of the row, scale by 0.8 into cell d
      (pixels of a neighbour's box are not masked);
  rA  crop the equal tenth and squeeze it into cell d.
Prints, per suffix and candidate, median and 5th percentile of the whole-atlas IoU, and the
per-digit median IoU. Writes example PNGs for one stem (png/<stem>_back_remap_<cand>.png).
Usage: python remap_test.py [max stems] [example stem, default a0150p1]"""
import glob
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glyph_cells import T17, T21, ink_mask, load_dds, load_ftex  # noqa: E402
import numpy as np  # noqa: E402
from PIL import Image  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
CELL = 2048 / 10
SLOTS = [0, 190] + [200 * d - 60 for d in range(2, 10)]
SLOTS_FIT = [0, 191, 344, 541, 740, 940, 1141, 1339, 1540, 1740]
# The stock PES 21 per-digit ink union (glyph_union.py), the region each digit is judged in.
REGION21 = [(0, 158), (200, 300), (346, 494), (546, 694), (740, 900), (946, 1094),
            (1144, 1296), (1343, 1498), (1546, 1696), (1746, 1896)]


def img(rgba):
    return Image.fromarray(rgba.astype(np.uint8), "RGBA")


def column_to_row(col, row_size, mode):
    """col: RGBA array (H tall, W wide); returns RGBA array of row_size (w, h)."""
    rw, rh = row_size
    f = rw / 2048
    src = img(col)
    ch = col.shape[0] / 10
    out = Image.new("RGBA", (rw, rh), tuple(int(v) for v in col[0, 0]))
    for d in range(10):
        box = (0, d * ch, col.shape[1], (d + 1) * ch)
        if mode == "A":
            w, h, x, y = CELL * f, rh, d * CELL * f, 0
        elif mode == "B":
            w, h = 128 * f, CELL * f
            x, y = d * CELL * f + (CELL * f - w) / 2, (rh - h) / 2
        else:
            slots = SLOTS if mode == "S" else SLOTS_FIT
            w, h, x, y = 160 * f, rh, slots[d] * f, 0
        cell = src.resize((round(w), round(h)), Image.BICUBIC, box=box)
        out.paste(cell, (round(x), round(y)))
    return np.asarray(out).astype(np.int32)


def row_to_column(row, col_size, mode):
    cw, chh = col_size
    f = chh / 2048
    src = img(row)
    out = Image.new("RGBA", (cw, chh), tuple(int(v) for v in row[0, 0]))
    for d in range(10):
        if mode == "rS":
            box = (SLOTS[d] * f, 0, (SLOTS[d] + 160) * f, row.shape[0])
        else:
            box = (d * CELL * f, 0, (d + 1) * CELL * f, row.shape[0])
        cell = src.resize((cw, round(CELL * f)), Image.BICUBIC, box=box)
        out.paste(cell, (0, round(d * CELL * f)))
    return np.asarray(out).astype(np.int32)


def iou(a, b):
    return (a & b).sum() / max(1, (a | b).sum())


def main():
    limit = int(sys.argv[1]) if len(sys.argv) > 1 and sys.argv[1] != "-" else None
    example = sys.argv[2] if len(sys.argv) > 2 else "a0150p1"
    s17 = {os.path.basename(p)[:-9] for p in glob.glob(os.path.join(T17, "*_back.dds"))}
    s21 = {os.path.basename(p)[:-10] for p in glob.glob(os.path.join(T21, "*_back.ftex"))}
    stems = sorted(s17 & s21)[:limit]
    if example not in stems:
        stems.append(example)
    forward, reverse = ("A", "B", "S", "Sf"), ("rS", "rA")
    scores = {}
    os.makedirs(os.path.join(HERE, "png"), exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        for stem in stems:
            for suffix in ("back", "chest", "leg"):
                p17 = os.path.join(T17, f"{stem}_{suffix}.dds")
                p21 = os.path.join(T21, f"{stem}_{suffix}.ftex")
                if not (os.path.exists(p17) and os.path.exists(p21)):
                    continue
                col, row = load_dds(p17), load_ftex(p21, tmp)
                m21, _ = ink_mask(row)
                m17, _ = ink_mask(col)
                f = row.shape[1] / 2048
                for mode in forward:
                    out = column_to_row(col, (row.shape[1], row.shape[0]), mode)
                    m, _ = ink_mask(out)
                    per = [iou(m[:, round(a * f):round(b * f)], m21[:, round(a * f):round(b * f)]) for a, b in REGION21]
                    scores.setdefault((suffix, mode), []).append((iou(m, m21), per))
                    if stem == example and suffix == "back":
                        img(out).save(os.path.join(HERE, "png", f"{stem}_back_remap_{mode}.png"))
                        print(f"example {stem}_back {mode}: IoU {iou(m, m21):.3f}")
                for mode in reverse:
                    out = row_to_column(row, (col.shape[1], col.shape[0]), mode)
                    m, _ = ink_mask(out)
                    ch = col.shape[0] / 10
                    per = [iou(m[round(d * ch):round((d + 1) * ch)], m17[round(d * ch):round((d + 1) * ch)]) for d in range(10)]
                    scores.setdefault((suffix, mode), []).append((iou(m, m17), per))
    print(f"stems {len(stems)}")
    for suffix in ("back", "chest", "leg"):
        print(f"== _{suffix}")
        for mode in forward + reverse:
            vals = scores.get((suffix, mode), [])
            whole = np.array([v for v, _ in vals])
            per = np.array([p for _, p in vals])
            print(f"   {mode:3s} n {len(whole)}  IoU median {np.median(whole):.3f} p5 {np.percentile(whole, 5):.3f}"
                  f"  per digit median " + " ".join(f"{x:.2f}" for x in np.median(per, 0)))


if __name__ == "__main__":
    main()
