"""Groups reports_<tag>.txt's findings of one code by kind: the `at` path's first folder and
its file extension (or "folder"), with the number of exports and up to four examples each.
The report is under `config.CENSUS_DIR`.
Usage: analyze.py <tag> <code> [substring that the input path must hold]
"""
import os
import re
import sys
from collections import defaultdict

import config

HERE = config.CENSUS_DIR
tag, code = sys.argv[1], sys.argv[2]
needle = sys.argv[3].lower() if len(sys.argv) > 3 else ""
LINE = re.compile(r"^- (?:(.*?): )?(Error|Warning|Info) (\S+)(?: at (.*?))?(?: \((.*)\))?$")

exports = defaultdict(set)
examples = defaultdict(list)
current = None
for line in open(os.path.join(HERE, f"reports_{tag}.txt"), encoding="utf-8"):
    line = line.rstrip("\n")
    if line.startswith("=== "):
        current = line[4:]
        continue
    match = LINE.match(line)
    if not match or match.group(3) != code or needle not in current.lower():
        continue
    at = match.group(4) or ""
    parts = at.split("/")
    extension = os.path.splitext(parts[-1])[1].lower() or "(none)"
    kind = f"{parts[0] if len(parts) > 1 else '(root)'} *{extension}"
    exports[kind].add(current)
    if len(examples[kind]) < 4:
        examples[kind].append(f"{current} :: {at} ({match.group(5) or ''})")
for kind in sorted(exports, key=lambda k: -len(exports[k])):
    print(f"{len(exports[kind]):4} exports  {kind}")
    for example in examples[kind]:
        print("        ", example)
