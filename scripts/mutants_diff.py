"""`just mutants-diff [base]`: mutation-test only the lines changed since `base`.

Runs `cargo mutants --in-diff` over `git diff <base>` (working tree and index against `base`,
default HEAD), so a review measures the tests of the code under review rather than the whole
workspace (CONTRIBUTING.md "Testing and verification", "Mutation runs"). The diff goes through
a file because the justfile forbids pipes and redirection in recipe lines.
"""

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main(argv: list[str]) -> int:
    base = argv[1] if len(argv) > 1 else "HEAD"
    diff = subprocess.run(
        ["git", "diff", base, "--", "crates"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    ).stdout
    if not diff.strip():
        print(f"mutants_diff: no changes under crates/ since {base}")
        return 0
    with tempfile.NamedTemporaryFile("w", suffix=".diff", delete=False, encoding="utf-8") as f:
        f.write(diff)
        diff_path = f.name
    command = ["cargo", "mutants", "--in-diff", diff_path, "--jobs", "2"]
    print("mutants_diff: " + " ".join(command))
    return subprocess.run(command, cwd=ROOT).returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv))
