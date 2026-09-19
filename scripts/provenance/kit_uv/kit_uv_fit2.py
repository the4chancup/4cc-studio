"""Per-bin median mapping 17->21 from position-map matching (tight 5 mm), clean variant pairs.
For each 20-px bin of u17 print median u21 (and count); same for v. Shape of the mapping is read
off directly: constant offset = shift, linearly growing offset = scale."""
import os, sys, glob
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kit_uv_fit import posmap, tris17, tris21, CELL, P17_32, P17_35, P21_35, P21_CM
import numpy as np

def matched(pm17, pm21, u_lo, u_hi, v_lo, v_hi, tol=0.005):
    sl = (slice(v_lo // CELL, v_hi // CELL), slice(u_lo // CELL, u_hi // CELL))
    def pts(pm):
        sub = pm[sl]; ok = ~np.isnan(sub[..., 0]); vv, uu = np.nonzero(ok)
        return np.stack([(uu + u_lo // CELL + 0.5) * CELL, (vv + v_lo // CELL + 0.5) * CELL], 1), sub[ok]
    uv17, p17 = pts(pm17); uv21, p21 = pts(pm21)
    idx = np.empty(len(p21), np.int64); dist = np.empty(len(p21))
    for i in range(0, len(p21), 2000):
        d = ((p21[i:i + 2000, None, :] - p17[None, :, :]) ** 2).sum(-1)
        idx[i:i + 2000] = d.argmin(1); dist[i:i + 2000] = np.sqrt(d.min(1))
    g = dist < tol
    return uv17[idx[g]], uv21[g], uv17, uv21

def bins(name, src, dst, axis, step=20):
    lab = 'uv'[axis]
    lo = int(src[:, axis].min() // step * step); hi = int(src[:, axis].max() // step * step + step)
    print(f'   {lab}17 bin      median {lab}21   offset   n')
    for b in range(lo, hi, step):
        m = (src[:, axis] >= b) & (src[:, axis] < b + step)
        if m.sum() >= 15:
            md = np.median(dst[m, axis]); print(f'   {b:5d}..{b+step:5d}   {md:8.1f}   {md-(b+step/2):+7.1f}   {m.sum()}')

def run(name, pm17, pm21, box):
    src, dst, uv17, uv21 = matched(pm17, pm21, *box)
    print(f'== {name}: 17 texels {len(uv17)} (u {uv17[:,0].min():.0f}..{uv17[:,0].max():.0f}, v {uv17[:,1].min():.0f}..{uv17[:,1].max():.0f})  21 texels {len(uv21)} (u {uv21[:,0].min():.0f}..{uv21[:,0].max():.0f}, v {uv21[:,1].min():.0f}..{uv21[:,1].max():.0f})  matched {len(src)}')
    bins(name, src, dst, 0); bins(name, src, dst, 1)

print('building maps...')
S17 = posmap(tris17([os.path.join(P17_32, 'socks_long.model')])); S21 = posmap(tris21([os.path.join(P21_CM, 'socks_long.fmdl')]))
run('socks_long LEFT', S17, S21, (0, 520, 600, 1200))
SH17 = posmap(tris17([os.path.join(P17_32, 'shirt_out.model')])); SH21 = posmap(tris21([os.path.join(P21_35, 'shirt_out.fmdl')]))
run('shirt_out CONTROL', SH17, SH21, (690, 1360, 30, 1880))
P17 = posmap(tris17(glob.glob(os.path.join(P17_35, 'pants_*.model')) + glob.glob(os.path.join(P17_32, 'pants_*_sub.model'))))
P21 = posmap(tris21(glob.glob(os.path.join(P21_35, 'pants_*.fmdl')) + glob.glob(os.path.join(P21_CM, 'pants_*sub*.fmdl'))))
run('pants all LEFT', P17, P21, (0, 700, 1100, 2000))
