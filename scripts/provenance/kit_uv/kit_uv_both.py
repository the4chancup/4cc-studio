"""Render uniform-model UVs of both engines over kit texture space and print per-mesh bounds.
Evidence only; PES17 .model via ModelFile, PES21 .fmdl via FmdlFile (19to16 converter libs)."""
import os, sys, glob
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile
from PIL import Image, ImageDraw

OUT = os.path.dirname(os.path.abspath(__file__))
S = 1024
P17U = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P17B = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\body'
P21U = os.path.join(OUT, 'pes21_uniform', 'Asset', 'model', 'character', 'uniform', 'nocloth', '#Win')
P21C = os.path.join(OUT, 'pes21_uniform', 'Asset', 'model', 'character', 'uniform', 'cloth', '#Win')
P21P = r'E:\PES2021\Data\dt32_g4_files\Asset\model\character\parts'

def meshes17(path):
    m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
    for mesh in m.meshes:
        name = getattr(mesh.material, 'name', None) or '?'
        yield name, mesh.vertices, mesh.faces

def meshes21(path):
    f = FmdlFile.FmdlFile(); f.readFile(path)
    for mesh in f.meshes:
        mi = mesh.materialInstance
        name = (mi.name if mi else '?') or '?'
        tex = ','.join(os.path.basename(t.filename) for t in (mi.textures if mi else []) if getattr(t, 'filename', None))
        yield f'{name} [{tex[:50]}]', mesh.vertices, mesh.faces

def render(tag, items):
    sheet = Image.new('RGB', (S, S), (0, 0, 0)); d = ImageDraw.Draw(sheet)
    palette = [(255,80,80),(80,255,80),(80,140,255),(255,255,80),(255,80,255),(80,255,255),(255,160,40),(180,100,255),(220,220,220),(120,255,160),(255,120,160),(160,200,80)]
    for i, (label, path, reader) in enumerate(items):
        col = palette[i % len(palette)]
        print(f'[{tag}] {label}')
        for name, verts, faces in reader(path):
            us = [v.uv[0].u for v in verts if v.uv]; vs = [v.uv[0].v for v in verts if v.uv]
            if not us:
                print(f'     {name:40s} no uv'); continue
            print(f'     {name:40s} n={len(verts):5d} u {min(us):.4f}..{max(us):.4f} v {min(vs):.4f}..{max(vs):.4f}   px u {min(us)*2048:5.0f}..{max(us)*2048:5.0f} v {min(vs)*2048:5.0f}..{max(vs)*2048:5.0f}')
            for face in faces:
                if all(v.uv for v in face.vertices):
                    d.polygon([(v.uv[0].u * S, v.uv[0].v * S) for v in face.vertices], outline=col)
        d.text((8, 8 + 12 * i), label, fill=col)
    sheet.save(os.path.join(OUT, f'kit_uv_{tag}.png'))

items17 = [(f, os.path.join(P17U, f), meshes17) for f in ['shirt_out.model', 'pants_out_sub.model', 'socks_long.model', 'socks_short.model']]
items17 += [(f'body/{f}', os.path.join(P17B, f), meshes17) for f in ['thigh_long.model', 'thigh_middle.model', 'thigh_short.model', 'arm.model', 'arm_half.model', 'arm_cap.model']]
render('17', items17)

items21 = [(f, os.path.join(P21U, f), meshes21) for f in ['shirt_out.fmdl', 'shirt.fmdl', 'pants_001.fmdl', 'pants_008.fmdl', 'pants_016.fmdl', 'sleeve_short_001.fmdl', 'sleeve_long_001.fmdl', 'cap_sleeve_short.fmdl']]
items21 += [(f'cloth/{f}', os.path.join(P21C, f), meshes21) for f in ['pants_001.fmdl', 'shirt_out.fmdl']]
items21 += [(f'parts/{f}', os.path.join(P21P, f), meshes21) for f in [r'naked\scenes\#Win naked_body.fmdl', r'torso\scenes\#Win torso.fmdl', r'torso\scenes\#Win torso_arm.fmdl']]
render('21', items21)
