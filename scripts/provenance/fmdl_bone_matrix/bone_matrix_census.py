"""Bone-matrix agreement census over every .model on E:\\PES2017: for each bone name that
occurs in several files, the max absolute component difference against the first occurrence,
and per file pair the max difference over shared bones. Read-only; prints."""
import os
import struct
import zlib
from collections import defaultdict


def unwrap(b):
    if b[3:8] == b"WESYS":
        return zlib.decompress(b[16:])
    return b


def sections(b):
    offs = struct.unpack_from("<11I", b, 36)
    order = sorted(range(11), key=lambda i: offs[i])
    out = {}
    for pos, i in enumerate(order):
        start = 24 + offs[i]
        end = 24 + offs[order[pos + 1]] if pos + 1 < 11 else len(b)
        out[i] = b[start:end]
    return out


def toc(sec, at=0):
    return struct.unpack_from("<3I", sec, at)


def bones(b):
    s = sections(b)
    s0, s5 = s[0], s[5]
    f0, c0, _ = toc(s0)
    if c0 == 0:
        return {}
    e0 = struct.unpack_from("<I", s0, f0)[0]
    ef, ec, es = toc(s0, e0)
    mats = []
    for r in range(ec):
        off, fmt, cnt = struct.unpack_from("<3I", s0, e0 + ef + r * es)
        assert fmt == 7 and cnt == 1
        mats.append(struct.unpack_from("<12f", s0, e0 + off))
    f5, c5, _ = toc(s5)
    names = []
    for r in range(c5):
        off = struct.unpack_from("<I", s5, f5 + 4 * r)[0]
        end = s5.index(b"\0", off)
        names.append(s5[off:end].decode())
    assert len(names) == len(mats), (len(names), len(mats))
    return dict(zip(names, mats))


files = []
for root, dirs, fs in os.walk(r"E:\PES2017"):
    if "Sider" in root:
        continue
    for f in fs:
        if f.lower().endswith(".model"):
            files.append(os.path.join(root, f))
print("files", len(files))

first = {}          # name -> (file, matrix)
worst = defaultdict(float)   # name -> max abs diff vs first
worst_file = {}
occurrences = defaultdict(int)
per_file = {}
for path in files:
    try:
        bs = bones(unwrap(open(path, "rb").read()))
    except Exception as e:  # noqa: BLE001 - census, report and go on
        print("FAIL", path, e)
        continue
    per_file[path] = bs
    for name, m in bs.items():
        occurrences[name] += 1
        if name not in first:
            first[name] = (path, m)
            continue
        d = max(abs(a - b) for a, b in zip(first[name][1], m))
        if d > worst[name]:
            worst[name] = d
            worst_file[name] = path

print("\n## distinct bone names", len(first))
buckets = defaultdict(int)
for name in first:
    if occurrences[name] < 2:
        buckets["single"] += 1
        continue
    d = worst[name]
    if d == 0:
        buckets["exact"] += 1
    elif d < 1e-5:
        buckets["<1e-5"] += 1
    elif d < 1e-4:
        buckets["<1e-4"] += 1
    elif d < 1e-3:
        buckets["<1e-3"] += 1
    elif d < 1e-2:
        buckets["<1e-2"] += 1
    else:
        buckets[">=1e-2"] += 1
for k, v in buckets.items():
    print(f"{v:6}  {k}")

print("\n## worst per bone name (sorted desc), top 40")
for name in sorted(worst, key=lambda n: -worst[n])[:40]:
    print(f"{worst[name]:.3e}  {occurrences[name]:5}  {name:32}  {os.path.relpath(worst_file[name], r'E:\PES2017')}")

print("\n## names with worst >= 1e-3, grouped by folder of the deviating file")
folders = defaultdict(list)
for name, d in worst.items():
    if d >= 1e-3:
        folders[os.path.dirname(os.path.relpath(worst_file[name], r"E:\PES2017"))].append((name, d))
for folder, items in sorted(folders.items()):
    print(folder, len(items), sorted(items, key=lambda x: -x[1])[:5])
