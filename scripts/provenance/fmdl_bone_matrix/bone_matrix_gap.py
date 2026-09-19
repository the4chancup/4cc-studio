"""The gap between float noise and real bind-pose differences: max difference under 1e-4 and
min difference at or above 1e-4, over every (bone, file) against the bone's first occurrence."""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from bone_matrix_census import bones, unwrap  # noqa: E402

files = []
for root, dirs, fs in os.walk(r"E:\PES2017"):
    if "Sider" in root:
        continue
    for f in fs:
        if f.lower().endswith(".model"):
            files.append(os.path.join(root, f))

first = {}
below = (0.0, None)
above = (1.0, None)
for path in files:
    for name, m in bones(unwrap(open(path, "rb").read())).items():
        if name not in first:
            first[name] = m
            continue
        d = max(abs(a - b) for a, b in zip(first[name], m))
        if d < 1e-4 and d > below[0]:
            below = (d, (name, path))
        if d >= 1e-4 and d < above[0]:
            above = (d, (name, path))
print("max noise below 1e-4:", below)
print("min real at/above 1e-4:", above)
