"""Census of where glyphs sit along the long axis, over every stock atlas of one game.

For each `_back`/`_chest`/`_leg` atlas: the ink runs along the long axis (glyph_runs.runs,
gap = length/400). Atlases with exactly ten runs are "clean" (one run per glyph); for those
the run starts, ends and centres are recorded as fractions of the length. Prints, per game
and suffix, how many atlases are clean, and per glyph the median and 5th/95th percentile of
start, centre and end fractions, plus the median ink extent across the short axis.
Writes glyph_census_<game>.csv. Usage: python glyph_census.py <17|21> [max files per suffix]"""
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


def main():
    game = sys.argv[1]
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else None
    folder, ext = (T17, ".dds") if game == "17" else (T21, ".ftex")
    out = csv.writer(open(os.path.join(HERE, f"glyph_census_{game}.csv"), "w", newline=""))
    out.writerow(["file", "w", "h", "runs", "glyph", "start", "end", "across0", "across1"])
    with tempfile.TemporaryDirectory() as tmp:
        for suffix in ("back", "chest", "leg"):
            files = sorted(glob.glob(os.path.join(folder, f"*_{suffix}{ext}")))[:limit]
            counts, starts, ends, across = {}, [], [], []
            for path in files:
                rgba = load_dds(path) if ext == ".dds" else load_ftex(path, tmp)
                mask, _ = ink_mask(rgba)
                h, w = mask.shape
                column = h > w
                length = h if column else w
                found = runs(mask.any(1) if column else mask.any(0), max(1, length // 400))
                counts[len(found)] = counts.get(len(found), 0) + 1
                cross = np.nonzero(mask.any(0) if column else mask.any(1))[0]
                short = w if column else h
                for g, (a, b) in enumerate(found):
                    out.writerow([os.path.basename(path), w, h, len(found), g, a / length, b / length,
                                  cross.min() / short if len(cross) else "", (cross.max() + 1) / short if len(cross) else ""])
                if len(found) == 10:
                    starts.append([a / length for a, _ in found])
                    ends.append([b / length for _, b in found])
                    across.append((cross.min() / short, (cross.max() + 1) / short))
            print(f"== PES{game} _{suffix}: {len(files)} files; runs per file: {sorted(counts.items())}")
            if not starts:
                continue
            s, e = np.array(starts), np.array(ends)
            c = (s + e) / 2
            print(f"   across (short axis) median {np.median([a for a, _ in across]):.3f}-{np.median([b for _, b in across]):.3f}")
            print("   glyph  start p5/med/p95        centre p5/med/p95       end p5/med/p95")
            for g in range(10):
                q = lambda v: "/".join(f"{x:.4f}" for x in np.percentile(v[:, g], [5, 50, 95]))  # noqa: E731
                print(f"   {g}      {q(s)}   {q(c)}   {q(e)}")


if __name__ == "__main__":
    main()
