"""The sock islands' pre-Fox -> Fox mapping along u, measured two ways, and the two-band fit.

A sock is a tube: v runs along the leg, u wraps around it. The two engines' sock meshes are
different meshes on bodies of different proportions, so matching texels by the nearest 3D point
(kit_uv_fit2.py) mixes the layout change with the bodies' difference. Two correspondences that
do not depend on the leg's thickness, length or place are used here instead, both per texel row
(one v):

- angle: the angle of a texel's body point around the leg's own axis, about the centre of the
  circle fitted to the row's ring. A pre-Fox texel maps to the Fox u with the same angle.
- arc: the fraction of the ring's 3D length between the row's first texel and this one. A
  pre-Fox texel maps to the Fox u at the same fraction.

Prints, per island (the right one mirrored onto the left's coordinates): each engine's seam
angles and turn, where its rows sit along the leg (whether v moved), each engine's share of the
ring per 16 px of u (its own texel density, no correspondence involved), the median offset
Fox u - pre-Fox u per 16-px bin for both correspondences with its range over the four quarters
of v, and the least-squares two-band map through the island's ends for each correspondence.
Last, with every variant pooled: the fitted meeting point of the two bands, and how far the
table's point, PES Master's and a single scale each sit from the pairs.
Usage: python kit_uv_sock_angle.py [variant ...]   (long, middle, short, noguard; default all)"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from kit_uv_fit import P17_32, P21_CM, tris17, tris21

R = 2048
CELL = 2  # px per cell of the position map
FIT_CELL = 4  # kit_uv_fit's triangles come in cells of this many px
U_HI, V_LO, V_HI = 520, 600, 1200  # the left sock island's box, px
# The seams, from the meshes' vertices (px): both ends of the tube's cut.
ENDS_17, ENDS_21 = (11.0, 438.0), (11.0, 369.0)
BIN = 16
# The point where the two bands meet: the one KIT_LAYOUT_REMAP uses, and the one read off PES
# Master's templates (docs/plans/team_compiler/pipeline.md), both checked against the pairs.
# Candidate maps, as (pre-Fox u, Fox u) points joined by straight lines: KIT_LAYOUT_REMAP's two
# bands, the two bands read off PES Master's templates (docs/plans/team_compiler/pipeline.md),
# and one scale from seam to seam. Each is checked against the pairs below.
TABLE = ((8, 8), (168, 128), (448, 376))
PES_MASTER = ((8, 8), (195, 137), (440, 380))
ONE_SCALE = ((ENDS_17[0], ENDS_21[0]), (ENDS_17[1], ENDS_21[1]))


def posmap(tris, mirror):
    """Texel -> body point for the left island's box, at CELL px per cell. With `mirror`, the
    right island is read as if it were the left one: u and the body's x both flipped."""
    w, h = U_HI // CELL, (V_HI - V_LO) // CELL
    pm = np.full((h, w, 3), np.nan)
    k = FIT_CELL / CELL
    flip = np.array([-1.0, 1.0, 1.0]) if mirror else np.ones(3)
    for t in tris:
        us = [p[0][0] * k for p in t]
        if mirror:
            us = [R / CELL - u for u in us]
        vs = [p[0][1] * k - V_LO / CELL for p in t]
        ps = [np.array(p[1]) * flip for p in t]
        (u0, u1, u2), (v0, v1, v2) = us, vs
        x0, x1 = int(max(0, min(us))), int(min(w - 1, max(us))) + 1
        y0, y1 = int(max(0, min(vs))), int(min(h - 1, max(vs))) + 1
        if x1 <= x0 or y1 <= y0:
            continue
        det = (v1 - v2) * (u0 - u2) + (u2 - u1) * (v0 - v2)
        if abs(det) < 1e-9:
            continue
        xs, ys = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
        l0 = ((v1 - v2) * (xs - u2) + (u2 - u1) * (ys - v2)) / det
        l1 = ((v2 - v0) * (xs - u2) + (u0 - u2) * (ys - v2)) / det
        l2 = 1 - l0 - l1
        inside = (l0 >= -1e-6) & (l1 >= -1e-6) & (l2 >= -1e-6)
        if not inside.any():
            continue
        pts = l0[..., None] * ps[0] + l1[..., None] * ps[1] + l2[..., None] * ps[2]
        pm[y0:y1, x0:x1][inside] = pts[inside]
    return pm


def circle_center(xy):
    """Least-squares circle through 2D points (Kasa's fit): it does not lean toward the side
    of the ring that holds more texels, as a centroid would."""
    a = np.column_stack([2 * xy[:, 0], 2 * xy[:, 1], np.ones(len(xy))])
    (cx, cy, _), *_ = np.linalg.lstsq(a, (xy**2).sum(1), rcond=None)
    return np.array([cx, cy])


def rows(pm):
    """Per texel row holding a whole ring: u px of each texel, its angle around the leg
    (unwrapped along u), its arc fraction from the first texel, the ring's length, and the
    row's place along the leg's axis."""
    ok = ~np.isnan(pm[..., 0])
    pts = pm[ok]
    mean = pts.mean(0)
    # The leg's axis: the direction the island's points spread most along.
    _, _, vt = np.linalg.svd(pts - mean, full_matrices=False)
    axis = vt[0] if vt[0][1] > 0 else -vt[0]
    front = np.array([0.0, 0.0, 1.0])
    e1 = front - front.dot(axis) * axis
    e1 /= np.linalg.norm(e1)
    e2 = np.cross(axis, e1)
    out = {}
    for j in range(pm.shape[0]):
        cols = np.nonzero(ok[j])[0]
        if len(cols) < 40 or (np.diff(cols) != 1).any():
            continue
        p = pm[j, cols]
        rel = p - mean
        xy = np.column_stack([rel.dot(e1), rel.dot(e2)])
        c = circle_center(xy)
        ang = np.unwrap(np.arctan2(xy[:, 1] - c[1], xy[:, 0] - c[0]))
        s = np.concatenate([[0.0], np.cumsum(np.linalg.norm(np.diff(p, axis=0), axis=1))])
        out[j] = {"u": (cols + 0.5) * CELL, "angle": ang, "arc": s / s[-1], "ring": s[-1],
                  "along": float(np.median(rel.dot(axis)))}
    return out


def describe(name, r):
    js = sorted(r)
    first = np.median([np.degrees(r[j]["angle"][0]) % 360 for j in js])
    turn = np.median([np.degrees(r[j]["angle"][-1] - r[j]["angle"][0]) for j in js])
    print(f"  {name}: {len(js)} rows, v {js[0] * CELL + V_LO}..{js[-1] * CELL + V_LO + CELL};"
          f" first texel at {first:.1f} deg, turn {turn:+.1f} deg; ring length median"
          f" {np.median([r[j]['ring'] for j in js]):.3f} m")
    shares = []
    for b in range(BIN, U_HI, BIN):
        row_shares = [np.interp(b + BIN, r[j]["u"], r[j]["arc"]) - np.interp(b, r[j]["u"], r[j]["arc"])
                      for j in js if r[j]["u"][0] <= b and r[j]["u"][-1] >= b + BIN]
        if len(row_shares) > 20:
            shares.append(f"{b}:{100 * np.median(row_shares):.2f}")
    print(f"    share of the ring per {BIN} px of u, percent: " + " ".join(shares))


def pairs(r17, r21, key):
    """(pre-Fox u, Fox u, v) of every pre-Fox texel, by equal `key` on the same row."""
    src, dst, vrow = [], [], []
    for j in sorted(set(r17) & set(r21)):
        a, b = r17[j], r21[j]
        k17, k21 = a[key], b[key]
        if key == "angle":
            k21 = k21 + 2 * np.pi * np.round((np.median(k17) - np.median(k21)) / (2 * np.pi))
        order = np.argsort(k21)
        inside = (k17 >= k21.min()) & (k17 <= k21.max())
        src.append(a["u"][inside])
        dst.append(np.interp(k17[inside], k21[order], b["u"][order]))
        vrow.append(np.full(inside.sum(), j * CELL + V_LO))
    return np.concatenate(src), np.concatenate(dst), np.concatenate(vrow)


def two_band(u, ub, fb):
    """The continuous two-band map through the island's ends and the point (ub, fb)."""
    (a0, a1), (b0, b1) = ENDS_17, ENDS_21
    return np.where(u <= ub, b0 + (fb - b0) * (u - a0) / (ub - a0), fb + (b1 - fb) * (u - ub) / (a1 - ub))


def fit(src, dst):
    """The (ub, fb) with the least squared error, ub over multiples of 4 px."""
    (a0, a1), (b0, b1) = ENDS_17, ENDS_21
    best = None
    for ub in range(40, 420, 4):
        lo = src <= ub
        # dst = base + fb * weight, linear in fb.
        weight = np.where(lo, (src - a0) / (ub - a0), 1 - (src - ub) / (a1 - ub))
        base = np.where(lo, b0 * (1 - (src - a0) / (ub - a0)), b1 * (src - ub) / (a1 - ub))
        fb = float(weight.dot(dst - base) / weight.dot(weight))
        err = float(np.sqrt(np.mean((two_band(src, ub, fb) - dst) ** 2)))
        if best is None or err < best[2]:
            best = (ub, fb, err)
    return best


def report(src, dst, vrow, label):
    quarters = np.linspace(vrow.min(), vrow.max() + 1, 5)
    print(f"  {label}: median Fox u - pre-Fox u per {BIN}-px bin of pre-Fox u (range over the quarters of v)")
    cells = []
    for b in range(0, U_HI, BIN):
        m = (src >= b) & (src < b + BIN)
        if m.sum() < 30:
            continue
        qs = [np.median(dst[mq] - src[mq]) for q in range(4)
              if (mq := m & (vrow >= quarters[q]) & (vrow < quarters[q + 1])).sum() >= 8]
        cells.append(f"{b}:{np.median(dst[m] - src[m]):+.0f}({min(qs):+.0f}..{max(qs):+.0f})")
    for i in range(0, len(cells), 9):
        print("    " + " ".join(cells[i:i + 9]))
    ub, fb, err = fit(src, dst)
    print(f"    two-band fit: ({ub}, {fb:.1f}), rms {err:.1f} px;"
          f" slopes {(fb - ENDS_21[0]) / (ub - ENDS_17[0]):.3f}, {(ENDS_21[1] - fb) / (ENDS_17[1] - ub):.3f}")
    return ub, fb


def residual(src, dst, points, label):
    xs, ys = zip(*points)
    e = np.interp(src, xs, ys) - dst
    return (f"{label}: rms {np.sqrt(np.mean(e**2)):.1f} px, 90% within {np.percentile(abs(e), 90):.1f} px,"
            f" max {abs(e).max():.1f} px")


def measure(variant, pooled, quiet=False):
    """Adds the variant's pairs to `pooled[(island, correspondence)]`, the right island's in
    the left one's coordinates."""
    t17 = list(tris17([os.path.join(P17_32, f"socks_{variant}.model")]))
    t21 = list(tris21([os.path.join(P21_CM, f"socks_{variant}.fmdl")]))
    if len(t21) < 100:
        # PES 21's socks_short.fmdl is a one-triangle stub.
        if not quiet:
            print(f"== socks_{variant}: the Fox model holds {len(t21)} triangle(s), skipped")
        return
    for island, mirror in (("left", False), ("right", True)):
        r17, r21 = rows(posmap(t17, mirror)), rows(posmap(t21, mirror))
        by_angle = pairs(r17, r21, "angle")
        by_arc = pairs(r17, r21, "arc")
        pooled.setdefault((island, "angle"), []).append(by_angle[:2])
        pooled.setdefault((island, "arc"), []).append(by_arc[:2])
        if quiet:
            continue
        print(f"== socks_{variant}, {island} island" + (" (mirrored)" if mirror else ""))
        describe("pre-Fox", r17)
        describe("Fox", r21)
        def frac(r):
            js = sorted(r)
            s = np.array([r[j]["along"] for j in js])
            return dict(zip(js, (s - s.min()) / (s.max() - s.min())))
        f17, f21 = frac(r17), frac(r21)
        print("  along the leg, fraction of the sock's length at v (pre-Fox/Fox): " + " ".join(
            f"{j * CELL + V_LO}:{f17[j]:.2f}/{f21[j]:.2f}" for j in sorted(set(f17) & set(f21))[::32]))
        report(*by_angle, "by angle")
        report(*by_arc, "by arc")


def pooled_pairs(variants=("long", "middle", "short", "noguard"), quiet=True):
    """{(island, correspondence): (pre-Fox u, Fox u)} over the variants, the right island's in
    the left one's coordinates."""
    pooled = {}
    for variant in variants:
        measure(variant, pooled, quiet)
    return {key: (np.concatenate([p[0] for p in parts]), np.concatenate([p[1] for p in parts]))
            for key, parts in pooled.items()}


def main():
    by = pooled_pairs(sys.argv[1:] or ("long", "middle", "short", "noguard"), quiet=False)
    every = (np.concatenate([p[0] for p in by.values()]), np.concatenate([p[1] for p in by.values()]))
    ub, fb, err = fit(*every)
    print(f"== every variant, both islands, both correspondences pooled: two-band fit ({ub}, {fb:.1f}),"
          f" rms {err:.1f} px")
    fitted = ((ENDS_17[0], ENDS_21[0]), (ub, fb), (ENDS_17[1], ENDS_21[1]))
    for name, points in (("the fit", fitted), ("the table", TABLE), ("PES Master", PES_MASTER), ("one scale", ONE_SCALE)):
        print(f"  {name} {points}")
        for key in ("angle", "arc"):
            both = [by[(island, key)] for island in ("left", "right")]
            print("    " + residual(np.concatenate([b[0] for b in both]), np.concatenate([b[1] for b in both]), points,
                                    f"against {key}"))


if __name__ == "__main__":
    main()
