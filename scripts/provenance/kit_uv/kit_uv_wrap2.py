"""Cylindrical mapping comparison, left-leg islands (u < 1024), all variants of each garment.
Socks: u <- angle, v <- height.  Pants: v <- angle, u <- height (island is sideways).
Prints binned means for both engines side by side so a scale/shift is readable."""
import os, sys, math, glob, collections
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile

P17_32 = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P17_35 = r'E:\PES2017\Data\dt35_win_files\common\character0\model\character\uniform\nocloth'
P21_35 = r'E:\PES2021\Data\dt35_g4_files\Asset\model\character\uniform\nocloth\#Win'
P21_CM = r'E:\PES2021\Data\common_package_fpk\Assets\pes16\model\character\common'

def verts17(paths):
    for path in paths:
        m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
        for mesh in m.meshes:
            for v in mesh.vertices:
                if v.uv:
                    p = v.position; yield (p.x, p.y, p.z, v.uv[0].u * 2048, v.uv[0].v * 2048)

def verts21(paths):
    for path in paths:
        f = FmdlFile.FmdlFile(); f.readFile(path)
        for mesh in f.meshes:
            for v in mesh.vertices:
                if v.uv:
                    p = v.position; yield (p.x, p.y, p.z, v.uv[0].u * 2048, v.uv[0].v * 2048)

def cyl(vs, u_lo, u_hi, v_lo, v_hi):
    vs = [t for t in vs if u_lo <= t[3] <= u_hi and v_lo <= t[4] <= v_hi]
    xs = [t[0] for t in vs]; zs = [t[2] for t in vs]
    cx, cz = (min(xs) + max(xs)) / 2, (min(zs) + max(zs)) / 2
    out = []
    for x, y, z, u, v in vs:
        out.append((math.degrees(math.atan2(z - cz, x - cx)) % 360, y, u, v))
    return out

def table(title, a17, a21, key_idx, key_step, key_lo, key_hi, val_idx, val_name):
    print(f'-- {title}: mean {val_name} per bin (17 | 21 | 21-17)')
    b17 = collections.defaultdict(list); b21 = collections.defaultdict(list)
    for t in a17: b17[int((t[key_idx] - key_lo) // key_step)].append(t[val_idx])
    for t in a21: b21[int((t[key_idx] - key_lo) // key_step)].append(t[val_idx])
    for b in range(int((key_hi - key_lo) / key_step)):
        x, y = b17.get(b), b21.get(b)
        lo = key_lo + b * key_step
        if x and y:
            mx, my = sum(x) / len(x), sum(y) / len(y)
            print(f'   {lo:7.2f}..{lo + key_step:7.2f}: {mx:7.1f} | {my:7.1f} | {my - mx:+7.1f}   (n {len(x)}/{len(y)})')
        else:
            print(f'   {lo:7.2f}..{lo + key_step:7.2f}: {"-" if not x else round(sum(x)/len(x))} | {"-" if not y else round(sum(y)/len(y))}')

print('################ SOCKS (all 4 variants, left island)')
s17 = cyl(list(verts17(glob.glob(os.path.join(P17_32, 'socks_*.model')))), 0, 500, 600, 1200)
s21 = cyl(list(verts21(glob.glob(os.path.join(P21_CM, 'socks_*.fmdl')))), 0, 500, 600, 1200)
print('   y range 17', min(t[1] for t in s17), max(t[1] for t in s17), ' 21', min(t[1] for t in s21), max(t[1] for t in s21))
table('socks u by angle', s17, s21, 0, 30, 0, 360, 2, 'u')
table('socks v by height', s17, s21, 1, 0.05, 0.0, 0.7, 3, 'v')

print('################ PANTS (all variants + sub, left island)')
p17 = cyl(list(verts17(glob.glob(os.path.join(P17_35, 'pants_*.model')) + glob.glob(os.path.join(P17_32, 'pants_*_sub.model')))), 0, 700, 1100, 2000)
p21 = cyl(list(verts21(glob.glob(os.path.join(P21_35, 'pants_*.fmdl')) + glob.glob(os.path.join(P21_CM, 'pants_*sub*.fmdl')))), 0, 700, 1100, 2000)
print('   y range 17', min(t[1] for t in p17), max(t[1] for t in p17), ' 21', min(t[1] for t in p21), max(t[1] for t in p21))
table('pants v by angle', p17, p21, 0, 30, 0, 360, 3, 'v')
table('pants u by height', p17, p21, 1, 0.05, 0.5, 1.15, 2, 'u')
