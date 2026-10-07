"""Same stock kit in both games: how each glyph's size and place changed between PES 17 and 21.

For every stem whose `_back`/`_chest`/`_leg` exist in both games, segments each atlas into
its ten glyphs by ink runs along the long axis (glyph_runs.runs; atlases without exactly ten
runs are skipped and counted), then per glyph takes the ink box in pixels: `along` (extent
on the long axis), `across` (extent on the short axis), so glyph width = across for the
column, along for the row; glyph height = along for the column, across for the row.

Per glyph and pair it records, with both atlases scaled to the 2048-long size (PES 17 x1 or
x2, PES 21 x1 or x2):
  * w17, h17, w21, h21 in px; sh = h21/h17, sw = w21/w17;
  * aspect ratio change (w21/h21)/(w17/h17): 1.0 means drawn in proportion, 1.28 means
    stretched with the cell (cell aspect 0.8 against 0.625);
  * y0, y1 of the glyph across the short axis of the PES 21 row as fractions of its height,
    and x0, x1 across the PES 17 column as fractions of its width;
  * the glyph's normalized box in a ten-equal-cell grid of each game (x0 x1 y0 y1).
Writes glyph_pairs.csv and prints medians and 5th/95th percentiles per suffix.
Usage: python glyph_pairs.py [max stems]"""
import csv
import glob
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glyph_cells import T17, T21, ink_mask, load_dds, load_ftex  # noqa: E402
from glyph_runs import runs  # noqa: E402
import numpy as np  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))


def glyph_boxes(rgba):
    """Ten (x0, x1, y0, y1) pixel boxes scaled to a 2048-long atlas, or None."""
    mask, _ = ink_mask(rgba)
    h, w = mask.shape
    column = h > w
    length = h if column else w
    k = 2048 / length
    found = runs(mask.any(1) if column else mask.any(0), max(1, length // 400))
    if len(found) != 10:
        return None, (w * k, h * k)
    boxes = []
    for a, b in found:
        part = mask[a:b, :] if column else mask[:, a:b]
        cross = np.nonzero(part.any(0) if column else part.any(1))[0]
        c0, c1 = cross.min(), cross.max() + 1
        box = (c0, c1, a, b) if column else (a, b, c0, c1)
        boxes.append(tuple(v * k for v in box))
    return boxes, (w * k, h * k)


def pct(values):
    return "/".join(f"{x:.3f}" for x in np.percentile(values, [5, 50, 95]))


def main():
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else None
    s17 = {os.path.basename(p)[:-9] for p in glob.glob(os.path.join(T17, "*_back.dds"))}
    s21 = {os.path.basename(p)[:-10] for p in glob.glob(os.path.join(T21, "*_back.ftex"))}
    stems = sorted(s17 & s21)[:limit]
    rows, skipped = [], 0
    with tempfile.TemporaryDirectory() as tmp:
        for stem in stems:
            for suffix in ("back", "chest", "leg"):
                p17 = os.path.join(T17, f"{stem}_{suffix}.dds")
                p21 = os.path.join(T21, f"{stem}_{suffix}.ftex")
                if not (os.path.exists(p17) and os.path.exists(p21)):
                    continue
                b17, (W17, H17) = glyph_boxes(load_dds(p17))
                b21, (W21, H21) = glyph_boxes(load_ftex(p21, tmp))
                if b17 is None or b21 is None:
                    skipped += 1
                    continue
                for g in range(10):
                    x0, x1, y0, y1 = b17[g]
                    u0, u1, v0, v1 = b21[g]
                    w17, h17, w21, h21 = x1 - x0, y1 - y0, u1 - u0, v1 - v0
                    c17 = (x0 / W17, x1 / W17, (y0 - g * 204.8) / 204.8, (y1 - g * 204.8) / 204.8)
                    c21 = ((u0 - g * 204.8) / 204.8, (u1 - g * 204.8) / 204.8, v0 / H21, v1 / H21)
                    rows.append([stem, suffix, g, w17, h17, w21, h21, *c17, *c21])
    with open(os.path.join(HERE, "glyph_pairs.csv"), "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["stem", "suffix", "glyph", "w17", "h17", "w21", "h21",
                    "c17x0", "c17x1", "c17y0", "c17y1", "c21x0", "c21x1", "c21y0", "c21y1"])
        w.writerows(rows)
    print(f"stems {len(stems)}, pairs skipped (not ten runs) {skipped}, glyph rows {len(rows)}")
    a = np.array([r[2:] for r in rows], dtype=float)
    suffixes = np.array([r[1] for r in rows])
    for suffix in ("back", "chest", "leg"):
        m = a[suffixes == suffix]
        g, w17, h17, w21, h21 = m[:, 0], m[:, 1], m[:, 2], m[:, 3], m[:, 4]
        not_one = g != 1  # the 1 is narrow and its width is a poor scale probe
        print(f"== _{suffix}: {len(m) // 10} pairs (p5/median/p95; sizes in px of a 2048-long atlas)")
        print(f"   h17 {pct(h17)}  h21 {pct(h21)}  height ratio h21/h17 {pct(h21 / h17)}")
        print(f"   w17 {pct(w17[not_one])}  w21 {pct(w21[not_one])}  width ratio w21/w17 (no 1s) {pct(w21[not_one] / w17[not_one])}")
        print(f"   aspect change (w21/h21)/(w17/h17), no 1s: {pct((w21 / h21)[not_one] / (w17 / h17)[not_one])}  (1 = proportional, 1.28 = stretched with cell)")
        names = ["x0", "x1", "y0", "y1"]
        print("   equal-cell normalized box  PES17: " + "  ".join(f"{n} {pct(m[:, 5 + i])}" for i, n in enumerate(names)))
        print("   equal-cell normalized box  PES21: " + "  ".join(f"{n} {pct(m[:, 9 + i])}" for i, n in enumerate(names)))


if __name__ == "__main__":
    main()
