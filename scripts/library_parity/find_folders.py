"""Lists every folder on the machine that looks like an old (AET) export root.

A folder is a candidate when, among its direct children, it holds at least two
of the markers `source.rs` `holds_old_export` knows (an OLD_FOLDERS folder, a
Note, a `refs.txt` / `refs list.txt`), or a Note or refs file alone with one
folder marker. Excluded: `config.EXCLUDED`. A candidate inside another candidate is
kept apart as nested. Writes folders.tsv (path, markers) and nested.tsv under
`config.CENSUS_DIR`, and prints the counts. Reads only. Usage: find_folders.py
"""
import json
import os
import urllib.parse
import urllib.request
from collections import defaultdict

import config

HERE = config.CENSUS_DIR
OLD_FOLDERS = config.OLD_FOLDERS
EXCLUDED = config.EXCLUDED


def query(search):
    url = (config.EVERYTHING + "?json=1&count=1000000&path_column=1&search="
           + urllib.parse.quote(search))
    with urllib.request.urlopen(url, timeout=120) as response:
        data = json.load(response)
    rows = [(entry["path"], entry["name"], entry["type"]) for entry in data["results"]]
    assert len(rows) == data["totalResults"], (search, len(rows), data["totalResults"])
    return rows


def excluded(path):
    lowered = path.lower() + "\\"
    return any(part in lowered for part in EXCLUDED)


markers = defaultdict(set)
for name in OLD_FOLDERS:
    for parent, child, kind in query(f'folder:wholefilename:"{name}"'):
        if kind == "folder":
            markers[parent].add(child.lower())
for search, label in [('file:wholefilename:"refs.txt"', "refs.txt"),
                      ('file:wholefilename:"refs list.txt"', "refs list.txt"),
                      ('file:"* note.txt"', "note")]:
    for parent, child, kind in query(search):
        if kind == "file":
            markers[parent].add(label)

candidates = {}
for parent, found in markers.items():
    if excluded(parent):
        continue
    files = found & {"refs.txt", "refs list.txt", "note"}
    folders = found - files
    if len(found) >= 2 or (files and folders) or ("refs.txt" in files or "refs list.txt" in files):
        candidates[parent] = found

def inside_export(path):
    """Whether `path` is part of another candidate's export: below one of its
    marker folders (a referee's `Players/x`, an `Other/` keepsake), or a
    referee's folder of a refs export, or a game asset tree or Red's templates."""
    lowered = path.lower()
    if "\\asset\\model\\character" in lowered or "\\templates" in lowered:
        return True
    for other, found in candidates.items():
        prefix = other.lower() + "\\"
        if other == path or not lowered.startswith(prefix):
            continue
        first = lowered[len(prefix):].split("\\")[0]
        if first in OLD_FOLDERS or found & {"refs.txt", "refs list.txt"}:
            return True
    return False


outer, nested = {}, {}
for path in sorted(candidates, key=str.lower):
    (nested if inside_export(path) else outer)[path] = candidates[path]


def write(name, table):
    os.makedirs(HERE, exist_ok=True)
    out = os.path.join(HERE, name)
    tmp = out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        for path in sorted(table, key=str.lower):
            handle.write(path + "\t" + ",".join(sorted(table[path])) + "\n")
    os.replace(tmp, out)


write("folders.tsv", outer)
write("nested.tsv", nested)
print(len(markers), "folders with a marker;", len(candidates), "candidates;",
      len(outer), "outermost,", len(nested), "nested")
