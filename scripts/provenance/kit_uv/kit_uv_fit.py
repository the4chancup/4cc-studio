"""Fit the pre-Fox -> Fox texture transform per island from the models themselves.
For each engine rasterize a position map (texel -> 3D body point) at 4-px cells; match each Fox
texel to the pre-Fox texel with the nearest body point; least-squares fit an axis-aligned affine
u' = a*u + b, v' = c*v + d; report parameters, residuals, and a sanity check on island edges."""
import os, sys, glob
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile
import numpy as np

CELL = 4; G = 2048 // CELL
P17_32 = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P17_35 = r'E:\PES2017\Data\dt35_win_files\common\character0\model\character\uniform\nocloth'
P21_35 = r'E:\PES2021\Data\dt35_g4_files\Asset\model\character\uniform\nocloth\#Win'
P21_CM = r'E:\PES2021\Data\common_package_fpk\Assets\pes16\model\character\common'

def tris17(paths):
    for path in paths:
        m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
        for mesh in m.meshes:
            for f in mesh.faces:
                if all(v.uv for v in f.vertices):
                    yield [((v.uv[0].u * G, v.uv[0].v * G), (v.position.x, v.position.y, v.position.z)) for v in f.vertices]

def tris21(paths):
    for path in paths:
        f = FmdlFile.FmdlFile(); f.readFile(path)
        for mesh in f.meshes:
            for face in mesh.faces:
                if all(v.uv for v in face.vertices):
                    yield [((v.uv[0].u * G, v.uv[0].v * G), (v.position.x, v.position.y, v.position.z)) for v in face.vertices]

def posmap(tris):
    pm = np.full((G, G, 3), np.nan, np.float64)
    for t in tris:
        (u0, v0), p0 = t[0]; (u1, v1), p1 = t[1]; (u2, v2), p2 = t[2]
        x0, x1 = int(max(0, min(u0, u1, u2))), int(min(G - 1, max(u0, u1, u2))) + 1
        y0, y1 = int(max(0, min(v0, v1, v2))), int(min(G - 1, max(v0, v1, v2))) + 1
        if x1 <= x0 or y1 <= y0:
            continue
        xs, ys = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
        det = (v1 - v2) * (u0 - u2) + (u2 - u1) * (v0 - v2)
        if abs(det) < 1e-9:
            continue
        l0 = ((v1 - v2) * (xs - u2) + (u2 - u1) * (ys - v2)) / det
        l1 = ((v2 - v0) * (xs - u2) + (u0 - u2) * (ys - v2)) / det
        l2 = 1 - l0 - l1
        inside = (l0 >= -0.02) & (l1 >= -0.02) & (l2 >= -0.02)
        if not inside.any():
            continue
        P = np.array([p0, p1, p2])
        pts = l0[..., None] * P[0] + l1[..., None] * P[1] + l2[..., None] * P[2]
        sub = pm[y0:y1, x0:x1]
        sub[inside] = pts[inside]
    return pm

def fit(name, pm17, pm21, u_lo, u_hi, v_lo, v_hi, mirror_x=False):
    sl = (slice(v_lo // CELL, v_hi // CELL), slice(u_lo // CELL, u_hi // CELL))
    def pts(pm):
        sub = pm[sl]; ok = ~np.isnan(sub[..., 0])
        vv, uu = np.nonzero(ok)
        return np.stack([(uu + u_lo // CELL + 0.5) * CELL, (vv + v_lo // CELL + 0.5) * CELL], 1), sub[ok]
    uv17, p17 = pts(pm17); uv21, p21 = pts(pm21)
    if mirror_x:
        p17 = p17.copy(); p17[:, 0] *= -1
    # nearest 17 texel for each 21 texel (chunked brute force)
    idx = np.empty(len(p21), np.int64); dist = np.empty(len(p21))
    for i in range(0, len(p21), 2000):
        d = ((p21[i:i + 2000, None, :] - p17[None, :, :]) ** 2).sum(-1)
        idx[i:i + 2000] = d.argmin(1); dist[i:i + 2000] = np.sqrt(d.min(1))
    good = dist < 0.01  # 1 cm body match
    src, dst = uv17[idx[good]], uv21[good]
    # axis-aligned affine per axis
    A = np.stack([src[:, 0], np.ones(len(src))], 1); (a, b), *_ = np.linalg.lstsq(A, dst[:, 0], rcond=None)
    A = np.stack([src[:, 1], np.ones(len(src))], 1); (c, d), *_ = np.linalg.lstsq(A, dst[:, 1], rcond=None)
    ru = dst[:, 0] - (a * src[:, 0] + b); rv = dst[:, 1] - (c * src[:, 1] + d)
    print(f'== {name}: texels 17={len(uv17)} 21={len(uv21)} matched(<1cm)={good.sum()} ({100*good.mean():.0f}%)')
    print(f'   u\' = {a:.4f}*u + {b:7.1f}     residual |ru| median {np.median(abs(ru)):.1f} px, 90% {np.percentile(abs(ru), 90):.1f} px')
    print(f'   v\' = {c:.4f}*v + {d:7.1f}     residual |rv| median {np.median(abs(rv)):.1f} px, 90% {np.percentile(abs(rv), 90):.1f} px')
    e17 = (uv17[:, 0].min(), uv17[:, 0].max(), uv17[:, 1].min(), uv17[:, 1].max()); e21 = (uv21[:, 0].min(), uv21[:, 0].max(), uv21[:, 1].min(), uv21[:, 1].max())
    print(f'   island 17 u {e17[0]:.0f}..{e17[1]:.0f} v {e17[2]:.0f}..{e17[3]:.0f}  -> mapped u {a*e17[0]+b:.0f}..{a*e17[1]+b:.0f} v {c*e17[2]+d:.0f}..{c*e17[3]+d:.0f}   | island 21 u {e21[0]:.0f}..{e21[1]:.0f} v {e21[2]:.0f}..{e21[3]:.0f}')
    # also a pure-scale-about-edge check and a pure-shift check for u
    for lbl, (aa, bb) in (('shift only', (1.0, np.median(dst[:, 0] - src[:, 0]))),):
        r = dst[:, 0] - (aa * src[:, 0] + bb); print(f'   [{lbl}] u\' = u + {bb:.1f}: median |ru| {np.median(abs(r)):.1f}, 90% {np.percentile(abs(r), 90):.1f}')

if __name__ == '__main__':
    print('building position maps...')
    S17 = posmap(tris17(glob.glob(os.path.join(P17_32, 'socks_*.model'))))
    S21 = posmap(tris21(glob.glob(os.path.join(P21_CM, 'socks_*.fmdl'))))
    fit('socks LEFT island', S17, S21, 0, 520, 600, 1200)
    fit('socks RIGHT island', S17, S21, 1500, 2048, 600, 1200)
    P17 = posmap(tris17(glob.glob(os.path.join(P17_35, 'pants_*.model')) + glob.glob(os.path.join(P17_32, 'pants_*_sub.model'))))
    P21 = posmap(tris21(glob.glob(os.path.join(P21_35, 'pants_*.fmdl')) + glob.glob(os.path.join(P21_CM, 'pants_*sub*.fmdl'))))
    fit('pants LEFT island', P17, P21, 0, 700, 1100, 2000)
    fit('pants RIGHT island', P17, P21, 1350, 2048, 1100, 2000)
    SH17 = posmap(tris17([os.path.join(P17_32, f) for f in ['shirt_out.model', 'shirt_in.model']]))
    SH21 = posmap(tris21([os.path.join(P21_35, f) for f in ['shirt_out.fmdl', 'shirt_in.fmdl']]))
    fit('shirt (control)', SH17, SH21, 690, 1360, 30, 1880)
    np.save(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'posmap_socks17.npy'), S17)
