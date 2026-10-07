"""Where a PES 17 digit cell lands in the PES 21 row, implied by same-team stock pairs.

Reads glyph_pairs.csv (glyph_pairs.py). Keeps glyph pairs drawn in the same proportions in
both games (aspect change (w21/h21)/(w17/h17) within 0.95-1.05, a proxy for "same font"),
and treats the PES 21 glyph as the PES 17 cell scaled uniformly by s = h21/h17 and moved:
  ox = u0_21 - s * x0_17   (left edge, in px of the 2048-long row, of the scaled cell)
  oy = v0_21 - s * y0_17   (top edge, in px of the 256-tall row, of the scaled cell)
where x0_17 is the glyph's ink left inside the 128-wide column and y0_17 its ink top inside
its 204.8-tall cell (both in px of the 2048-long atlas). Also the same fit for the right
and bottom edges (ox' = u1 - s*x1, oy' = v1 - s*y1), which agree with ox, oy when the
scale is uniform. Prints per digit and suffix: n, median s, ox, ox', oy, oy' and their
5th/95th percentiles. Usage: python slot_fit.py"""
import csv
import os

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
CELL17, WIDTH17 = 204.8, 128.0


def main():
    rows = list(csv.DictReader(open(os.path.join(HERE, "glyph_pairs.csv"))))
    for suffix in ("back", "chest", "leg"):
        print(f"== _{suffix} (px of the 2048-long atlas; PES 21 row is 256 tall)")
        print("   d    n   s p5/p50/p95          ox(left) p5/p50/p95     ox'(right)          oy(top)             oy'(bottom)")
        for g in range(10):
            vals = []
            for r in rows:
                if r["suffix"] != suffix or int(r["glyph"]) != g:
                    continue
                w17, h17, w21, h21 = (float(r[k]) for k in ("w17", "h17", "w21", "h21"))
                if not 0.95 <= (w21 / h21) / (w17 / h17) <= 1.05:
                    continue
                s = h21 / h17
                x0, x1 = float(r["c17x0"]) * WIDTH17, float(r["c17x1"]) * WIDTH17
                y0, y1 = float(r["c17y0"]) * CELL17, float(r["c17y1"]) * CELL17
                u0 = float(r["c21x0"]) * CELL17 + g * CELL17
                u1 = float(r["c21x1"]) * CELL17 + g * CELL17
                v0, v1 = float(r["c21y0"]) * 256, float(r["c21y1"]) * 256
                vals.append((s, u0 - s * x0, u1 - s * x1, v0 - s * y0, v1 - s * y1))
            if not vals:
                continue
            a = np.array(vals)
            q = lambda c: "/".join(f"{x:7.1f}" if c else f"{x:.3f}" for x in np.percentile(a[:, c], [5, 50, 95]))  # noqa: E731
            print(f"   {g} {len(a):4d}   {q(0)}   {q(1)}   {q(2)}   {q(3)}   {q(4)}")


if __name__ == "__main__":
    main()
