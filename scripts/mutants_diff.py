"""`just mutants-diff [base]`: mutation-test only the lines changed since `base`.

Runs `cargo mutants --in-diff` over `git diff <base>` (working tree and index against `base`,
default HEAD), so a review measures the tests of the code under review rather than the whole
workspace (CONTRIBUTING.md "Testing and verification", "Mutation runs"). The diff goes through
a file because the justfile forbids pipes and redirection in recipe lines.

A run whose mutants would cost more than a few minutes locally splits with
`STUDIO_MUTANTS_REMOTE` like `just mutants` does (see `mutants.py`): the
estimate comes from the per-crate per-mutant costs this machine has measured
(`mutants.estimate_seconds`), so a pes_savefile diff splits while a one-line
`fpc` fix stays local.
"""

import json
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

import mutants

ROOT = Path(__file__).resolve().parent.parent


def main(argv: list[str]) -> int:
    base = argv[1] if len(argv) > 1 else "HEAD"
    # Bytes, not text: the file must be LF on both sides (text mode writes
    # CRLF on Windows).
    diff = subprocess.run(
        ["git", "diff", base, "--", "crates"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout
    if not diff.strip():
        print(f"mutants_diff: no changes under crates/ since {base}")
        return 0
    with tempfile.NamedTemporaryFile("wb", suffix=".diff", delete=False) as f:
        f.write(diff)
        diff_path = f.name
    # The mutant list, for the printed line and the local-run estimate. Each
    # entry's `file` names the crate (`crates/<group>/<crate>/...`).
    listed = subprocess.run(
        ["cargo", "mutants", "--in-diff", diff_path, "--list", "--json"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout
    # A diff with no mutable lines prints nothing at all (an "INFO" line on
    # stderr), not an empty array.
    found = json.loads(listed) if listed.strip() else []
    crates = [mutants.mutant_crate(m["file"]) for m in found]
    counts = Counter(crates)
    estimated = mutants.estimate_seconds(crates)
    names = ", ".join(
        f"{crate}: {count}" for crate, count in sorted(counts.items())
    )
    print(
        f"mutants_diff: {len(found)} mutants ({names}), "
        f"estimated {estimated / 60:.0f} min on this machine"
    )
    host = mutants.remote_host()
    if host is not None and estimated >= mutants.SPLIT_THRESHOLD_SECONDS:
        return mutants.split(["--in-diff", diff_path], host)
    print("mutants_diff: running here", flush=True)
    command = ["cargo", "mutants", "--in-diff", diff_path, "--jobs", "2"]
    print("mutants_diff: " + " ".join(command), flush=True)
    code = subprocess.run(command, cwd=ROOT).returncode
    mutants.update_cost_cache()
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv))
