"""Like-for-like UV comparison of kit garments: PES17 nocloth/*.model vs PES21 common_package_fpk
pes16/common/*.fmdl (socks, pants_sub) + PES21 dt35 uniform/nocloth (shirt, pants_NNN).
Prints per-mesh UV bounds in 2048-px units and renders both layouts. Evidence only."""
import os, sys
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile
from PIL import Image, ImageDraw

OUT = os.path.dirname(os.path.abspath(__file__))
S = 1024
P17 = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P21C = r'E:\PES2021\Data\common_package_fpk\Assets\pes16\model\character\common'
P21U = os.path.join(OUT, 'pes21_uniform', 'Asset', 'model', 'character', 'uniform', 'nocloth', '#Win')

def meshes17(path):
    m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
    for mesh in m.meshes:
        yield '?', mesh.vertices, mesh.faces

def meshes21(path):
    f = FmdlFile.FmdlFile(); f.readFile(path)
    for mesh in f.meshes:
        mi = mesh.materialInstance
        yield (mi.name if mi and mi.name else '?'), mesh.vertices, mesh.faces

def render(tag, items):
    sheet = Image.new('RGB', (S, S), (0, 0, 0)); d = ImageDraw.Draw(sheet)
    palette = [(255,80,80),(80,255,80),(80,140,255),(255,255,80),(255,80,255),(80,255,255),(255,160,40),(180,100,255),(220,220,220),(120,255,160)]
    for i, (label, path, reader) in enumerate(items):
        col = palette[i % len(palette)]
        print(f'[{tag}] {label}')
        for name, verts, faces in reader(path):
            us = [v.uv[0].u for v in verts if v.uv]; vs = [v.uv[0].v for v in verts if v.uv]
            if not us:
                print(f'     {name:20s} no uv'); continue
            print(f'     {name:20s} n={len(verts):5d}  px u {min(us)*2048:6.1f}..{max(us)*2048:6.1f}  v {min(vs)*2048:6.1f}..{max(vs)*2048:6.1f}')
            for face in faces:
                if all(v.uv for v in face.vertices):
                    d.polygon([(v.uv[0].u * S, v.uv[0].v * S) for v in face.vertices], outline=col)
        d.text((8, 8 + 12 * i), label, fill=col)
    sheet.save(os.path.join(OUT, f'kit_uv_cmp_{tag}.png'))

names = ['socks_long', 'socks_middle', 'socks_short', 'socks_noguard', 'pants_in_sub', 'pants_out_sub', 'shirt_out', 'shirt_in']
items17 = [(n, os.path.join(P17, n + '.model'), meshes17) for n in names]
items21 = [(n, os.path.join(P21C, n + '.fmdl'), meshes21) for n in names if n.startswith(('socks', 'pants'))]
items21 += [(f'nocloth/{n}', os.path.join(P21U, n + '.fmdl'), meshes21) for n in ['shirt_out', 'pants_001', 'pants_016']]
render('17', items17)
render('21', items21)
