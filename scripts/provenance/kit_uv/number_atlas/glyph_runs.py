"""Where the glyphs really sit along an atlas's long axis, without assuming ten equal cells.

Projects the ink mask (glyph_cells.ink_mask) onto the long axis and prints every run of
ink-bearing lines (start, end, centre in pixels and as a fraction of the length), merged
across gaps of up to `gap` px so a glyph with a hole does not split. Also prints the ink's
extent across the short axis.
Usage: python glyph_runs.py <stem> [suffix ...]"""
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glyph_cells import T17, T21, ink_mask, load_dds, load_ftex  # noqa: E402
import numpy as np  # noqa: E402


def runs(line, gap):
    idx = np.nonzero(line)[0]
    if len(idx) == 0:
        return []
    out, start, prev = [], idx[0], idx[0]
    for i in idx[1:]:
        if i - prev > gap:
            out.append((start, prev + 1))
            start = i
        prev = i
    out.append((start, prev + 1))
    return out


def describe(game, rgba):
    mask, rule = ink_mask(rgba)
    h, w = mask.shape
    column = h > w
    length = h if column else w
    along = mask.any(1) if column else mask.any(0)
    across = mask.any(0) if column else mask.any(1)
    cross = np.nonzero(across)[0]
    print(f"  {game}: {w}x{h} ink by {rule}; across {cross.min()}-{cross.max() + 1} of {w if column else h}")
    for a, b in runs(along, max(1, length // 400)):
        print(f"    {a:5d}-{b:5d}  len {b - a:4d}  centre {(a + b) / 2:7.1f} = {(a + b) / 2 / length:.4f}")


def main():
    stem = sys.argv[1]
    with tempfile.TemporaryDirectory() as tmp:
        for suffix in sys.argv[2:] or ["back", "chest", "leg"]:
            print(f"== {stem}_{suffix}")
            describe("17", load_dds(os.path.join(T17, f"{stem}_{suffix}.dds")))
            describe("21", load_ftex(os.path.join(T21, f"{stem}_{suffix}.ftex"), tmp))


if __name__ == "__main__":
    main()
