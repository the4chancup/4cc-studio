"""`red_parity`'s classes over every `jobs/<n>/parity.tsv` a library run's rows file names
(under `config.LIBRARY_DIR`, the jobs folder `LIBRARY_JOBS` as for `attribute.py`): per
class, the number of lines and of exports, up to `examples` example lines (export, side, Red's
path or ours, detail), and per export its class counts and Red's time.

Usage: tally.py <rows_<tag>.tsv>... [--examples N]
"""
import argparse
import os
from collections import Counter, defaultdict

import config

HERE = config.LIBRARY_DIR
JOBS = os.environ.get("LIBRARY_JOBS", "jobs")
EQUAL = {"exact", "fmdl_equal", "ftex_equal", "dds_equal", "dds_reencoded_equal", "mtl_equal", "xml_equal",
         "uniparam_equal", "fpk_equal", "cpk_equal", "relocated", "relocated_id", "renamed_model",
         "dropped_fpk_xml", "studio_record"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+")
    parser.add_argument("--examples", type=int, default=4)
    args = parser.parse_args()
    rows = {}
    for file in args.files:
        for line in open(file, encoding="utf-8"):
            fields = line.rstrip("\n").split("\t")
            rows[int(fields[0])] = fields
    lines_per_class, exports_per_class = Counter(), defaultdict(set)
    examples = defaultdict(list)
    print("per export: n, version, Red exit, Red seconds, ours, classes")
    for index in sorted(rows):
        _n, path, version, red_code, red_seconds, _cpks, _up, _outcome, _seconds, summary = rows[index]
        # The compile's exit and Error codes from its log, past the log's three header lines.
        log = os.path.join(HERE, JOBS, str(index), "compile.txt")
        outcome = "-"
        if os.path.isfile(log):
            lines = open(log, encoding="utf-8").read().splitlines()
            errors = sorted({line.split(" Error ")[1].split()[0] for line in lines[3:] if " Error " in line})
            outcome = f"exit {lines[1].split()[1]}: " + (",".join(errors) or "-")
        print(f"  {index:4} PES {version} red exit {red_code} {red_seconds}s | ours {outcome[:90]} | {summary}")
        report = os.path.join(HERE, JOBS, str(index), "parity.tsv")
        if not os.path.isfile(report):
            continue
        for line in open(report, encoding="utf-8"):
            side, red, ours, class_, detail = line.rstrip("\n").split("\t")
            lines_per_class[class_] += 1
            exports_per_class[class_].add(index)
            if len(examples[class_]) < args.examples:
                examples[class_].append(f"[{index}] {side} {red or ours} {detail[:220]}")
    print("\nclass: lines, exports (equal classes first)")
    ordered = sorted(lines_per_class, key=lambda c: (c not in EQUAL, -lines_per_class[c]))
    for class_ in ordered:
        print(f"{class_}: {lines_per_class[class_]}, {len(exports_per_class[class_])}")
        if class_ not in EQUAL:
            for example in examples[class_]:
                print("    ", example)
    # The one-sided classes by what the path is, per export.
    print("\none-sided lines by kind: class, kind: lines (exports: lines each)")
    kinds = defaultdict(lambda: defaultdict(Counter))
    for index in sorted(rows):
        report = os.path.join(HERE, JOBS, str(index), "parity.tsv")
        if not os.path.isfile(report):
            continue
        for line in open(report, encoding="utf-8"):
            side, red, ours, class_, detail = line.rstrip("\n").split("\t")
            if class_ in ("red_only", "ours_only", "cpk_member_missing", "cpk_member_ours_only", "fpk_member_missing", "fpk_member_ours_only"):
                kinds[class_][kind_of(red or ours)][index] += 1
    for class_ in sorted(kinds):
        for kind, per_export in sorted(kinds[class_].items(), key=lambda item: -sum(item[1].values())):
            spread = ", ".join(f"{index}: {count}" for index, count in sorted(per_export.items()))
            print(f"{class_}, {kind}: {sum(per_export.values())} ({spread})")


def kind_of(path):
    lower = path.lower()
    name = lower.rsplit("/", 1)[-1]
    if "/boots/" in lower:
        return "boots package"
    if "/glove/" in lower:
        return "gloves package"
    if "/face/real/" in lower and (".cpk/" in lower or lower.endswith(".cpk")):
        return "face CPK member" if ".cpk/" in lower else "face CPK"
    if "/face/real/" in lower:
        return "face package"
    if "/uniform/texture/" in lower:
        return "kit texture"
    if "/uniform/team/referee/" in lower or "refereeappearance" in lower:
        return "referee template"
    if "/uniform/team/" in lower and name.endswith("_realuni.bin"):
        return "kit config"
    if name in ("unicolor.bin", "teamcolor.bin", "uniformparameter.bin"):
        return "bins"
    if "/render/symbol/player/" in lower:
        return "portrait"
    if "/render/symbol/flag/" in lower:
        return "logo"
    if "/common/" in lower and (name.endswith(".ftex") or name.endswith(".dds")):
        return "common texture"
    if "/common/" in lower:
        return "common other"
    return "other: " + name.rsplit(".", 1)[-1]


if __name__ == "__main__":
    main()
