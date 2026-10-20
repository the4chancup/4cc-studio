"""Classes every .zip, .7z and .rar Everything indexes by what its listing holds.

The root rule copies `source.rs`: the archive's top level holds an old-export
marker (`holds_old_export`), or its one top folder does, with no file beside it
(`root_prefix`); OS artifacts left out. Classes:
  export  - the upgrader's rule finds an export root (`root` column: "" or the wrapper)
  pack    - no root by that rule, but two or more top-level folders (at depth 1 or 2) hold markers
  other   - nothing an old export holds
  failed  - 7-Zip could not list it (the reason)
Excluded: `config.EXCLUDED`; `config.SKIPPED` counted, not listed. Writes
archives.tsv (path, size, class, detail) under `config.CENSUS_DIR` and prints the
tally. Reads only. Usage: find_archives.py
"""
import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request
from collections import Counter
from concurrent.futures import ThreadPoolExecutor

import config

HERE = config.CENSUS_DIR
SEVEN_ZIP = config.SEVEN_ZIP
OLD_FOLDERS = set(config.OLD_FOLDERS)
EXCLUDED = config.EXCLUDED
SKIPPED = config.SKIPPED
OS_ARTIFACTS = {"thumbs.db", "desktop.ini", ".ds_store"}


def query(search):
    url = (config.EVERYTHING + "?json=1&count=1000000&path_column=1&size_column=1&search="
           + urllib.parse.quote(search))
    with urllib.request.urlopen(url, timeout=120) as response:
        data = json.load(response)
    assert len(data["results"]) == data["totalResults"]
    return [(os.path.join(entry["path"], entry["name"]), int(entry.get("size") or 0))
            for entry in data["results"] if entry["type"] == "file"]


def is_artifact(path):
    parts = path.split("/")
    name = parts[-1].lower()
    return (parts[0] == "__MACOSX" or name in OS_ARTIFACTS or name.startswith("._"))


def listing(path):
    """(files, folders) as /-joined paths, or raise RuntimeError with 7-Zip's reason."""
    run = subprocess.run([SEVEN_ZIP, "l", "-slt", "-ba", "-p", path],
                         capture_output=True, timeout=600)
    if run.returncode != 0:
        tail = (run.stderr or run.stdout).decode("utf-8", "replace").strip().splitlines()
        raise RuntimeError(tail[-1] if tail else f"exit {run.returncode}")
    files, folders = [], set()
    entry = {}
    for line in run.stdout.decode("utf-8", "replace").splitlines() + [""]:
        if line.strip() == "":
            if "Path" in entry:
                name = entry["Path"].replace("\\", "/").strip("/")
                if entry.get("Folder") == "+" or entry.get("Attributes", "").startswith("D"):
                    folders.add(name)
                else:
                    files.append(name)
            entry = {}
            continue
        key, _, value = line.partition(" = ")
        entry[key] = value
    for name in files + list(folders):
        parts = name.split("/")
        for depth in range(1, len(parts)):
            folders.add("/".join(parts[:depth]))
    files = [name for name in files if not is_artifact(name)]
    folders = {name for name in folders if not is_artifact(name)}
    return files, folders


def holds(files, folders, prefix):
    """Whether the level below `prefix` holds an old-export marker."""
    def direct(names):
        for name in names:
            if name.startswith(prefix):
                rest = name[len(prefix):]
                if rest and "/" not in rest:
                    yield rest.lower()
    return (any(name.endswith(" note.txt") or name in ("refs.txt", "refs list.txt")
                for name in direct(files))
            or any(name in OLD_FOLDERS for name in direct(folders)))


def classify(item):
    path, size = item
    try:
        files, folders = listing(path)
    except Exception as error:  # a listing failure is a class of its own
        return path, size, "failed", str(error)
    if holds(files, folders, ""):
        return path, size, "export", ""
    top_folders = sorted(name for name in folders if "/" not in name)
    top_files = [name for name in files if "/" not in name]
    if len(top_folders) == 1 and not top_files and holds(files, folders, top_folders[0] + "/"):
        return path, size, "export", top_folders[0]
    roots = [name for name in folders if name.count("/") <= 1
             and holds(files, folders, name + "/")]
    if len(roots) >= 2:
        return path, size, "pack", f"{len(roots)} roots"
    if roots:
        return path, size, "other", f"one nested root: {roots[0]}"
    return path, size, "other", ""


def main():
    items = []
    for extension in ("zip", "7z", "rar"):
        items += [(path, size) for path, size in query(f"ext:{extension}")
                  if not any(part in path.lower() for part in EXCLUDED)]
    skipped = [item for item in items if any(item[0].lower().startswith(p) for p in SKIPPED)]
    items = [item for item in items if item not in skipped]
    print(len(items), "archives,", len(skipped), "skipped (SKIPPED trees)", flush=True)
    rows = []
    with ThreadPoolExecutor(max_workers=8) as pool:
        for count, row in enumerate(pool.map(classify, items), 1):
            rows.append(row)
            if count % 250 == 0:
                print(count, "listed", flush=True)
    os.makedirs(HERE, exist_ok=True)
    out = os.path.join(HERE, "archives.tsv")
    tmp = out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        for path, size, kind, detail in sorted(rows, key=lambda row: row[0].lower()):
            handle.write(f"{path}\t{size}\t{kind}\t{detail}\n")
    os.replace(tmp, out)
    tally = Counter((os.path.splitext(row[0])[1].lower(), row[2]) for row in rows)
    for key, count in sorted(tally.items()):
        print(key, count)


if __name__ == "__main__":
    sys.exit(main())
