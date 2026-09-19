"""Render PES17 uniform model UVs (nocloth/*.model) over texture space. Evidence only."""
import os, sys
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile
from PIL import Image, ImageDraw

OUT = os.path.dirname(os.path.abspath(__file__))
SRC = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
S = 1024

def uv_bounds(model):
    per_mesh = []
    for mesh in model.meshes:
        us, vs = [], []
        for v in mesh.vertices:
            if v.uv:
                us.append(v.uv[0].u); vs.append(v.uv[0].v)
        mat = getattr(mesh.material, 'name', None) or str(mesh.material)[:30]
        per_mesh.append((mat, len(mesh.vertices), min(us), max(us), min(vs), max(vs)))
    return per_mesh

files = sorted(f for f in os.listdir(SRC) if f.endswith('.model'))
colors = [(255,80,80),(80,255,80),(80,120,255),(255,255,80),(255,80,255),(80,255,255),(255,160,40),(160,80,255),(200,200,200),(120,255,160)]
sheet = Image.new('RGB', (S, S), (0,0,0))
d = ImageDraw.Draw(sheet)
for i, f in enumerate(files):
    ps = ModelFile.ParserSettings()
    m, _w = ModelFile.readModelFile(os.path.join(SRC, f), ps)
    print(f)
    for mat, nv, u0, u1, v0, v1 in uv_bounds(m):
        print(f'   mat={mat:30s} verts={nv:5d}  u {u0:.4f}..{u1:.4f}  v {v0:.4f}..{v1:.4f}  (px u {u0*2048:.0f}..{u1*2048:.0f}, v {v0*2048:.0f}..{v1*2048:.0f})')
    col = colors[i % len(colors)]
    for mesh in m.meshes:
        for face in mesh.faces:
            pts = [(v.uv[0].u * S, v.uv[0].v * S) for v in face.vertices]
            d.polygon(pts, outline=col)
    d.text((8, 8 + 12 * i), f, fill=col)
sheet.save(os.path.join(OUT, 'kit_uv_17.png'))
