"""`just mutants-diff [base]`: mutation-test only the lines changed since `base`.

Runs `cargo mutants --in-diff` over the diff from `base` (default HEAD) to the working tree,
untracked files included, so a review measures the tests of the code under review rather than the whole
workspace (CONTRIBUTING.md "Testing and verification", "Mutation runs"). The diff goes through
a file because the justfile forbids pipes and redirection in recipe lines.

A run whose mutants would cost more than a few minutes locally splits with
`STUDIO_MUTANTS_REMOTE` like `just mutants` does (see `mutants.py`): the
estimate is the baseline plus the per-crate per-mutant costs this machine has
measured (`mutants.estimate_seconds`), so a pes_savefile diff splits while a
one-line `fpc` fix stays local. A diff holding a mutant of a crate in
`mutants.LOCAL_ONLY_CRATES` never splits.
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
    # Against the whole working tree as a tree object, not `git diff <base>`,
    # which leaves out untracked files: a new module's mutants would silently
    # go unmeasured unless the tree was staged first (found at 3.8).
    # Bytes, not text: the file must be LF on both sides (text mode writes
    # CRLF on Windows).
    diff = subprocess.run(
        ["git", "diff", base, mutants.working_tree(), "--", "crates"],
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
        f"estimated {estimated:.0f} s on this machine"
    )
    host = mutants.remote_host()
    local_only = sorted(mutants.LOCAL_ONLY_CRATES.intersection(counts))
    if host is not None and estimated >= mutants.SPLIT_THRESHOLD_SECONDS and not local_only:
        return mutants.split(["--in-diff", diff_path], host)
    if host is not None and local_only:
        print(f"mutants_diff: {', '.join(local_only)} is local-only (LOCAL_ONLY_CRATES)")
    print("mutants_diff: running here", flush=True)
    print(f"mutants_diff: cargo mutants --in-diff {diff_path}", flush=True)
    code = mutants.local_mutants(["--in-diff", diff_path]).returncode
    mutants.update_cost_cache()
    return mutants.report_killed(code)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
