"""Union of every stock atlas's ink per digit, from glyph_census_<game>.csv (clean atlases only).

Prints per suffix and digit: min start, max end (fractions of the long axis, and px of a
2048-long atlas), the free gap to the next digit's min start, and the median centre. The
union is the narrowest window a sampler could use per digit without clipping any stock glyph.
Also prints the short-axis ink union. Usage: python glyph_union.py <17|21>"""
import csv
import os
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    game = sys.argv[1]
    rows = list(csv.DictReader(open(os.path.join(HERE, f"glyph_census_{game}.csv"))))
    by = defaultdict(list)
    across = defaultdict(list)
    for r in rows:
        if r["runs"] != "10":
            continue
        suffix = r["file"].rsplit("_", 1)[1].split(".")[0]
        by[(suffix, int(r["glyph"]))].append((float(r["start"]), float(r["end"])))
        across[suffix].append((float(r["across0"]), float(r["across1"])))
    for suffix in ("back", "chest", "leg"):
        a = across[suffix]
        print(f"== PES{game} _{suffix}: short-axis ink union {min(x for x, _ in a):.4f}-{max(y for _, y in a):.4f}")
        for g in range(10):
            spans = by[(suffix, g)]
            lo, hi = min(s for s, _ in spans), max(e for _, e in spans)
            centres = sorted((s + e) / 2 for s, e in spans)
            nxt = min(s for s, _ in by[(suffix, g + 1)]) if g < 9 else 1.0
            print(f"   {g}: {lo:.4f}-{hi:.4f} ({lo * 2048:6.0f}-{hi * 2048:6.0f} px)  gap to next {(nxt - hi) * 2048:6.0f} px"
                  f"  median centre {centres[len(centres) // 2]:.4f}  n {len(spans)}")


if __name__ == "__main__":
    main()
