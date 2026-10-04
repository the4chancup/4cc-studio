"""Why the shorts islands need no re-layout: the earlier "shift" was the Fox body's height.

kit_uv_fit2.py matched shorts texels across engines by the nearest 3D point and found the Fox
texels 36 to 60 px lower along u. On the shorts, u runs along the body's height (hem at low u,
waist at high u; v wraps around the leg), and the Fox body stands a few centimetres taller at
the hip, so a nearest-point match pairs each Fox texel with a pre-Fox texel higher up the
garment whatever the layout is. This script separates the two with measurements that do not
compare 3D positions across engines:

- the garment's own landmarks along u, per engine: the hem (lowest u any shorts model covers),
  the crotch notch (where the island's outline narrows along v) and the waist (highest u);
- the fraction of the garment's height (hem 0, waist 1) at each u, per engine;
- the angle around the leg at each v, per engine.

It also prints the height difference at equal u converted to px, which reproduces the earlier
"shift".
Usage: python kit_uv_pants_height.py"""
import glob
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from kit_uv_fit import P17_32, P17_35, P21_35, P21_CM, tris17, tris21

CELL = 4  # px per cell; kit_uv_fit's triangles come in these cells
U_HI, V_LO, V_HI = 700, 1100, 2000  # the left shorts island's box, px


def texels(tris):
    """(u px, v px, x, y, z) of every cell centre a triangle covers, left island only."""
    out = {}
    for t in tris:
        (u0, v0), p0 = t[0]
        (u1, v1), p1 = t[1]
        (u2, v2), p2 = t[2]
        det = (v1 - v2) * (u0 - u2) + (u2 - u1) * (v0 - v2)
        if abs(det) < 1e-9:
            continue
        for yy in range(int(min(v0, v1, v2)), int(max(v0, v1, v2)) + 1):
            for xx in range(int(min(u0, u1, u2)), int(max(u0, u1, u2)) + 1):
                cu, cv = xx + 0.5, yy + 0.5
                if not (cu * CELL < U_HI and V_LO <= cv * CELL < V_HI):
                    continue
                l0 = ((v1 - v2) * (cu - u2) + (u2 - u1) * (cv - v2)) / det
                l1 = ((v2 - v0) * (cu - u2) + (u0 - u2) * (cv - v2)) / det
                l2 = 1 - l0 - l1
                if l0 < -1e-6 or l1 < -1e-6 or l2 < -1e-6:
                    continue
                p = l0 * np.array(p0) + l1 * np.array(p1) + l2 * np.array(p2)
                out[(xx, yy)] = (cu * CELL, cv * CELL, p[0], p[1], p[2])
    return np.array(list(out.values()))


def landmarks(a):
    u, v = a[:, 0], a[:, 1]
    top_rows = a[v < v.min() + 16]
    return u.min() - CELL / 2, top_rows[:, 0].max() + CELL / 2, u.max() + CELL / 2


sets = {
    "pre-Fox": texels(tris17(sorted(glob.glob(os.path.join(P17_35, "pants_*.model")))
                             + sorted(glob.glob(os.path.join(P17_32, "pants_*_sub.model"))))),
    "Fox": texels(tris21(sorted(glob.glob(os.path.join(P21_35, "pants_*.fmdl")))
                         + sorted(glob.glob(os.path.join(P21_CM, "pants_*sub*.fmdl"))))),
}
height = {}
for name, a in sets.items():
    u, v, x, y, z = a.T
    hem, notch, waist = landmarks(a)
    print(f"== {name}, every shorts model, left island: {len(a)} cells; v {v.min() - 2:.0f}..{v.max() + 2:.0f}")
    print(f"   corr(u, height) {np.corrcoef(u, y)[0, 1]:+.3f}, corr(v, height) {np.corrcoef(v, y)[0, 1]:+.3f}")
    print(f"   landmarks along u, px: hem {hem:.0f}, crotch notch {notch:.0f}, waist {waist:.0f}")
    bins = [b for b in range(0, 672, 32) if ((u >= b) & (u < b + 32)).sum() > 20]
    med = {b: float(np.median(y[(u >= b) & (u < b + 32)])) for b in bins}
    lo, hi = min(med.values()), max(med.values())
    height[name] = med
    print("   fraction of the garment's height per 32-px bin of u: "
          + " ".join(f"{b}:{(med[b] - lo) / (hi - lo):.2f}" for b in bins))
    ang = np.degrees(np.arctan2(z - z.mean(), x - x.mean()))
    print("   angle around the leg per 64-px bin of v, deg: "
          + " ".join(f"{b}:{np.median(ang[(v >= b) & (v < b + 64)]):+.0f}"
                     for b in range(1152, 1984, 64) if ((v >= b) & (v < b + 64)).sum() > 20))

common = sorted(set(height["pre-Fox"]) & set(height["Fox"]))
slope = (height["pre-Fox"][common[-1]] - height["pre-Fox"][common[1]]) / (common[-1] - common[1])
print(f"== height at equal u: pre-Fox rises {1000 * slope:.3f} mm per px of u")
print("   Fox minus pre-Fox, as mm and as the px of u that height is worth: "
      + " ".join(f"{b}:{1000 * (height['Fox'][b] - height['pre-Fox'][b]):+.0f}mm"
                 f"({(height['Fox'][b] - height['pre-Fox'][b]) / slope:+.0f}px)" for b in common[1:]))
