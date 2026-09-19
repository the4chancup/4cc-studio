"""Histogram of every (bone, file) matrix difference against the bone's first occurrence, in
log10 buckets, over the non-Sider .model files; plus the same restricted to character/d (kit
parts, the merge's real inputs). Read-only; prints."""
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(__file__))
from bone_matrix_census import bones, unwrap  # noqa: E402  (reuses the parsers; census prints once on import)

files = []
for root, dirs, fs in os.walk(r"E:\PES2017"):
    if "Sider" in root:
        continue
    for f in fs:
        if f.lower().endswith(".model"):
            files.append(os.path.join(root, f))


def bucket(d):
    if d == 0:
        return "0"
    return f"1e{math.floor(math.log10(d))}"


def run(label, paths):
    first = {}
    hist = defaultdict(int)
    examples = defaultdict(list)
    for path in paths:
        bs = bones(unwrap(open(path, "rb").read()))
        for name, m in bs.items():
            if name not in first:
                first[name] = m
                continue
            d = max(abs(a - b) for a, b in zip(first[name], m))
            b = bucket(d)
            hist[b] += 1
            if len(examples[b]) < 3:
                examples[b].append((name, os.path.basename(os.path.dirname(path)) + "/" + os.path.basename(path)))
    print(f"\n## {label}: {len(paths)} files")
    for b in sorted(hist, key=lambda k: -1 if k == "0" else int(k[2:])):
        print(f"{hist[b]:7}  {b:6}  {examples[b]}")


run("all", files)
run("character/d only", [p for p in files if "\\character\\d\\" in p])
run("character/face only", [p for p in files if "\\character\\face\\" in p])
