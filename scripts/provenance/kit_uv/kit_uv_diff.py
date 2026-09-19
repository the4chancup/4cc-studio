"""Rasterize UV coverage of each kit garment class for PES17 and PES21, report per-island bounding
boxes (2048-px units) and write diff images (red = 17 only, green = 21 only, grey = both).
Evidence only, for the pre-Fox/Fox kit layout decision."""
import os, sys, glob, collections
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile
from PIL import Image, ImageDraw
import numpy as np

OUT = os.path.dirname(os.path.abspath(__file__))
R = 2048
P17_32 = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P17_35 = r'E:\PES2017\Data\dt35_win_files\common\character0\model\character\uniform\nocloth'
P21_35 = r'E:\PES2021\Data\dt35_g4_files\Asset\model\character\uniform\nocloth\#Win'
P21_CM = r'E:\PES2021\Data\common_package_fpk\Assets\pes16\model\character\common'

def tris17(path):
    m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
    for mesh in m.meshes:
        for f in mesh.faces:
            if all(v.uv for v in f.vertices):
                yield [(v.uv[0].u * R, v.uv[0].v * R) for v in f.vertices]

def tris21(path):
    f = FmdlFile.FmdlFile(); f.readFile(path)
    for mesh in f.meshes:
        for face in mesh.faces:
            if all(v.uv for v in face.vertices):
                yield [(v.uv[0].u * R, v.uv[0].v * R) for v in face.vertices]

def mask(paths, reader):
    im = Image.new('L', (R, R), 0); d = ImageDraw.Draw(im); n = 0
    for p in paths:
        for t in reader(p):
            d.polygon(t, fill=255, outline=255); n += 1
    return np.asarray(im) > 0, n

def islands(m, min_px=64):
    """Connected components (8-neighbour) on a 4x downsample; returns bboxes in full-res px."""
    s = m.reshape(R // 4, 4, R // 4, 4).any(axis=(1, 3))
    h, w = s.shape; seen = np.zeros_like(s); out = []
    for y0 in range(h):
        for x0 in range(w):
            if not s[y0, x0] or seen[y0, x0]:
                continue
            stack = [(y0, x0)]; seen[y0, x0] = True; ys = []; xs = []
            while stack:
                y, x = stack.pop(); ys.append(y); xs.append(x)
                for dy in (-1, 0, 1):
                    for dx in (-1, 0, 1):
                        ny, nx = y + dy, x + dx
                        if 0 <= ny < h and 0 <= nx < w and s[ny, nx] and not seen[ny, nx]:
                            seen[ny, nx] = True; stack.append((ny, nx))
            if len(ys) * 16 >= min_px:
                out.append((min(xs) * 4, min(ys) * 4, max(xs) * 4 + 4, max(ys) * 4 + 4, len(ys) * 16))
    return sorted(out)

classes = {
    'shirt':     ([os.path.join(P17_32, f) for f in ['shirt_out.model', 'shirt_in.model', 'shirt_out_tight.model', 'shirt_in_tight.model']],
                  [os.path.join(P21_35, f) for f in ['shirt_out.fmdl', 'shirt_in.fmdl', 'shirt_out_tight.fmdl', 'shirt_in_tight.fmdl', 'shirt.fmdl']]),
    'sleeves':   (glob.glob(os.path.join(P17_35, 'sleeve_*.model')), glob.glob(os.path.join(P21_35, 'sleeve_*.fmdl')) + glob.glob(os.path.join(P21_35, 'cap_sleeve_*.fmdl'))),
    'collars':   (glob.glob(os.path.join(P17_35, 'collar_*.model')), glob.glob(os.path.join(P21_35, 'collar_*.fmdl'))),
    'pants':     (glob.glob(os.path.join(P17_35, 'pants_*.model')), glob.glob(os.path.join(P21_35, 'pants_*.fmdl'))),
    'pants_sub': (glob.glob(os.path.join(P17_32, 'pants_*_sub.model')), glob.glob(os.path.join(P21_CM, 'pants_*sub*.fmdl'))),
    'socks':     (glob.glob(os.path.join(P17_32, 'socks_*.model')), glob.glob(os.path.join(P21_CM, 'socks_*.fmdl'))),
}

for name, (p17, p21) in classes.items():
    m17, n17 = mask(p17, tris17); m21, n21 = mask(p21, tris21)
    print(f'== {name}: 17 files={len(p17)} tris={n17} | 21 files={len(p21)} tris={n21}')
    for tag, m in (('17', m17), ('21', m21)):
        for (x0, y0, x1, y1, area) in islands(m):
            print(f'   {tag} island  u {x0:5d}..{x1:5d}  v {y0:5d}..{y1:5d}   ({(x1-x0)}x{(y1-y0)}, {area} px)')
    only17 = m17 & ~m21; only21 = m21 & ~m17; both = m17 & m21
    print(f'   coverage 17={m17.sum()} 21={m21.sum()} both={both.sum()} only17={only17.sum()} only21={only21.sum()}')
    rgb = np.zeros((R, R, 3), np.uint8)
    rgb[both] = (110, 110, 110); rgb[only17] = (230, 60, 60); rgb[only21] = (60, 220, 60)
    Image.fromarray(rgb).resize((1024, 1024), Image.BOX).save(os.path.join(OUT, f'kit_diff_{name}.png'))
