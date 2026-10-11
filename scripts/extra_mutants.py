"""`just extra-mutants <crate>`: run the mutants `cargo-mutants` lacks (worklog 5.0b).

`crates/dev/extra_mutants` lists two kinds: a dropped `if` guard (the condition replaced
with `false`) and one element removed from a literal list a `matches!` or a separator
method decides by. cargo-mutants rewrites operators and return values but neither of
those, so this script applies each listed patch in a copy of the working tree and lets
`cargo test -p <crate>` decide whether the suite catches it. Outcomes land in
`extra_mutants.out/` (gitignored): `outcomes.tsv` has every mutant, `missed.txt` the
survivors. `--list` only prints the mutants and their counts, without running anything.
"""

import json
import os
import shutil
import signal
import subprocess
import sys
import tarfile
import time
from collections import Counter
from pathlib import Path

import mutants

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "extra_mutants.out"
LISTER = ["cargo", "run", "-q", "--release", "-p", "extra_mutants", "--", "list"]
# Each mutant's `cargo test` may take up to this multiple of the unmutated baseline.
TIMEOUT_FACTOR = 3
MIN_TIMEOUT_SECONDS = 120


def crate_dir(name: str) -> Path:
    """The named workspace crate's directory, from `cargo metadata`."""
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    ).stdout
    for package in json.loads(out)["packages"]:
        if package["name"] == name:
            return Path(package["manifest_path"]).resolve().parent
    raise SystemExit(f"extra_mutants: no workspace crate named {name}")


def list_mutants(directory: Path) -> list[list[str]]:
    """The lister's rows (`path`, `start`, `end`, `kind`, `line`, `replacement`,
    `description`), run on the crate's sources under `directory`."""
    out = subprocess.run(
        [*LISTER, str(directory)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if out.returncode != 0:
        raise SystemExit(f"extra_mutants: the lister failed:\n{out.stderr}")
    rows = [line.split("\t") for line in out.stdout.splitlines() if line]
    for row in rows:
        if len(row) != 7:
            raise SystemExit(f"extra_mutants: malformed lister row: {row}")
    return rows


def extract_tree(tree: str, dest: Path) -> None:
    """Extract git tree `tree` into `dest` (created empty)."""
    dest.mkdir(parents=True)
    proc = subprocess.Popen(
        ["git", "archive", "--format=tar", tree],
        cwd=ROOT,
        stdout=subprocess.PIPE,
    )
    try:
        with tarfile.open(fileobj=proc.stdout, mode="r|") as archive:
            archive.extractall(dest, filter="data")
    except BaseException:
        proc.kill()
        proc.wait()
        raise
    if proc.wait() != 0:
        raise RuntimeError(f"git archive {tree} failed")


def kill_tree(proc: subprocess.Popen) -> None:
    """Kill `proc` and every process it started (cargo's rustc and test children)."""
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True
        )
    else:
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except ProcessLookupError:
            pass
    proc.wait()


def cargo_test(crate: str, cwd: Path, timeout: float | None) -> tuple[int | None, str]:
    """`cargo test -p <crate>` in `cwd` at below-normal priority, output merged.
    Returns (exit code, output); exit code is None when the timeout killed the
    process tree."""
    kwargs = mutants.below_normal()
    if os.name != "nt":
        kwargs["start_new_session"] = True
    proc = subprocess.Popen(
        ["cargo", "test", "-p", crate],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
        **kwargs,
    )
    try:
        out, _ = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        kill_tree(proc)
        return None, ""
    return proc.returncode, out


def baseline_seconds(crate: str, cwd: Path) -> float:
    """The unmutated `cargo test` wall time in `cwd`; a failing baseline ends the run."""
    start = time.monotonic()
    code, out = cargo_test(crate, cwd, timeout=None)
    if code != 0:
        print("extra_mutants: the unmutated baseline fails; last lines:")
        print("\n".join(out.splitlines()[-30:]))
        raise SystemExit(1)
    return time.monotonic() - start


def apply_patch(path: Path, start: int, end: int, replacement: str, removed: str) -> None:
    """Replace `data[start:end]` with `replacement`, checking the range holds the text
    the lister described (so a drifted file cannot silently patch somewhere else)."""
    data = path.read_bytes()
    if not (0 <= start <= end <= len(data)):
        raise AssertionError(f"{path}: mutant byte range {start}..{end} out of bounds")
    actual = " ".join(data[start:end].decode("utf-8").split())
    if actual != removed:
        raise AssertionError(
            f"{path}: the byte range holds {actual!r}, the lister described {removed!r}"
        )
    patched = data[:start] + replacement.encode("utf-8") + data[end:]
    if patched == data:
        raise AssertionError(f"{path}: mutant at {start}..{end} changes nothing")
    path.write_bytes(patched)


def write_out(path: Path, text: str) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_text(text, encoding="utf-8", newline="\n")
    os.replace(temp, path)


def main(argv: list[str]) -> int:
    args = argv[1:]
    list_only = "--list" in args
    args = [arg for arg in args if arg != "--list"]
    if len(args) != 1:
        print("usage: extra_mutants.py <crate> [--list]")
        return 2
    crate = args[0]
    directory = crate_dir(crate)
    if list_only:
        rows = list_mutants(directory)
        for row in rows:
            print("\t".join(row))
        kinds = Counter(row[3] for row in rows)
        counts = ", ".join(f"{kind} {n}" for kind, n in sorted(kinds.items()))
        print(f"{len(rows)} mutants ({counts})")
        return 0

    crate_rel = directory.relative_to(ROOT)
    tree = mutants.working_tree()
    copy_dir = mutants.LOCAL_TMPDIR / f"extra-mutants-{os.getpid()}"
    if copy_dir.exists():
        # A leftover of an interrupted run under a reused pid: remove it rather than
        # test against its stale files.
        shutil.rmtree(copy_dir)
    wall_start = time.monotonic()
    try:
        extract_tree(tree, copy_dir)
        rows = list_mutants(copy_dir / crate_rel)
        baseline = baseline_seconds(crate, copy_dir)
        timeout = max(MIN_TIMEOUT_SECONDS, TIMEOUT_FACTOR * baseline)
        print(
            f"extra_mutants: baseline {baseline:.0f} s, timeout {timeout:.0f} s, "
            f"{len(rows)} mutants",
            flush=True,
        )
        outcomes: list[tuple[str, list[str]]] = []
        for i, row in enumerate(rows, 1):
            rel, start, end, kind, line, replacement, description = row
            # A mutant path must stay inside the copy: an absolute or escaping one would
            # patch the real working tree.
            rel_path = Path(rel)
            if rel_path.is_absolute() or ".." in rel_path.parts:
                raise RuntimeError(f"extra_mutants: mutant path escapes the copy: {rel}")
            removed = description.rsplit(" | removed: ", 1)[-1]
            target = copy_dir / crate_rel / rel_path
            original = target.read_bytes()
            try:
                apply_patch(
                    target, int(start), int(end), replacement, removed
                )
                code, out = cargo_test(crate, copy_dir, timeout)
            finally:
                target.write_bytes(original)
            if code is None:
                outcome = "timeout"
            elif code == 0:
                outcome = "missed"
            elif "could not compile" in out:
                outcome = "unviable"
            else:
                outcome = "caught"
            outcomes.append((outcome, row))
            print(f"[{i}/{len(rows)}] {outcome}: {rel}:{line} {description}", flush=True)
        OUT_DIR.mkdir(exist_ok=True)
        write_out(
            OUT_DIR / "outcomes.tsv",
            "".join(
                outcome + "\t" + "\t".join(row) + "\n" for outcome, row in outcomes
            ),
        )
        write_out(
            OUT_DIR / "missed.txt",
            "".join(
                f"{row[0]}:{row[4]}: {row[6]}\n"
                for outcome, row in outcomes
                if outcome == "missed"
            ),
        )
        per_outcome = Counter(outcome for outcome, _ in outcomes)
        per_kind = Counter(row[3] for _, row in outcomes)
        print(
            "outcomes: "
            + ", ".join(f"{name} {n}" for name, n in sorted(per_outcome.items()))
        )
        print(
            "per kind: "
            + ", ".join(f"{name} {n}" for name, n in sorted(per_kind.items()))
        )
        print(f"wall time: {time.monotonic() - wall_start:.0f} s")
        return 0
    finally:
        if copy_dir.exists():
            try:
                shutil.rmtree(copy_dir)
            except OSError as error:
                print(f"extra_mutants: could not remove {copy_dir}: {error}")


if __name__ == "__main__":
    sys.exit(main(sys.argv))
