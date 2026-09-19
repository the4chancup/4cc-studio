"""How does the leg wrap into the sock / pants island? For each engine: per vertex (x,y,z,u,v);
print position ranges and, in 12 angular bins around the leg axis, the mean u (px) of the LEFT
island (u < 1024). If 21 = affine(17) it is a squeeze; if bins go missing it is a crop."""
import os, sys, math, collections
sys.path.insert(0, r'C:\Data\4cc\Tools_4cc\4cc-aet-converter-19to16\Engines\lib')
import ModelFile, FmdlFile

P17_32 = r'E:\PES2017\Data\dt32_win_files\common\character1\model\character\uniform\nocloth'
P17_35 = r'E:\PES2017\Data\dt35_win_files\common\character0\model\character\uniform\nocloth'
P21_35 = r'E:\PES2021\Data\dt35_g4_files\Asset\model\character\uniform\nocloth\#Win'
P21_CM = r'E:\PES2021\Data\common_package_fpk\Assets\pes16\model\character\common'

def verts17(path):
    m, _ = ModelFile.readModelFile(path, ModelFile.ParserSettings())
    for mesh in m.meshes:
        for v in mesh.vertices:
            if v.uv:
                p = v.position; yield (p.x, p.y, p.z, v.uv[0].u * 2048, v.uv[0].v * 2048)

def verts21(path):
    f = FmdlFile.FmdlFile(); f.readFile(path)
    for mesh in f.meshes:
        for v in mesh.vertices:
            if v.uv:
                p = v.position; yield (p.x, p.y, p.z, v.uv[0].u * 2048, v.uv[0].v * 2048)

def report(label, vs, u_lo, u_hi, v_lo, v_hi):
    vs = [t for t in vs if u_lo <= t[3] <= u_hi and v_lo <= t[4] <= v_hi]
    xs, ys, zs = [t[0] for t in vs], [t[1] for t in vs], [t[2] for t in vs]
    print(f'-- {label}: n={len(vs)}  x {min(xs):.3f}..{max(xs):.3f}  y {min(ys):.3f}..{max(ys):.3f}  z {min(zs):.3f}..{max(zs):.3f}')
    cx, cz = (min(xs) + max(xs)) / 2, (min(zs) + max(zs)) / 2
    bins = collections.defaultdict(list)
    for x, y, z, u, v in vs:
        a = math.degrees(math.atan2(z - cz, x - cx)) % 360
        bins[int(a // 30)].append(u)
    print('   angle bin -> mean u / min u / max u:')
    for b in range(12):
        us = bins.get(b, [])
        if us:
            print(f'     {b*30:3d}-{b*30+30:3d}: {sum(us)/len(us):7.1f}  {min(us):7.1f} {max(us):7.1f}  (n={len(us)})')
        else:
            print(f'     {b*30:3d}-{b*30+30:3d}: (none)')
    # v vs height
    top = [t for t in vs if t[1] > max(ys) - 0.01]; bot = [t for t in vs if t[1] < min(ys) + 0.01]
    print(f'   v at top of mesh ~{sum(t[4] for t in top)/len(top):.0f}   v at bottom ~{sum(t[4] for t in bot)/len(bot):.0f}')

print('=========== SOCKS (left island u<1024)')
report('17 socks_long', list(verts17(os.path.join(P17_32, 'socks_long.model'))), 0, 1024, 600, 1200)
report('21 socks_long', list(verts21(os.path.join(P21_CM, 'socks_long.fmdl'))), 0, 1024, 600, 1200)
print('=========== PANTS main+sub (left island u<1024)')
v17 = list(verts17(os.path.join(P17_35, 'pants_001.model'))) + list(verts17(os.path.join(P17_32, 'pants_out_sub.model')))
v21 = list(verts21(os.path.join(P21_35, 'pants_001.fmdl'))) + list(verts21(os.path.join(P21_CM, 'pants_out_sub.fmdl')))
report('17 pants_001+out_sub', v17, 0, 1024, 1100, 2000)
report('21 pants_001+out_sub', v21, 0, 1024, 1100, 2000)
