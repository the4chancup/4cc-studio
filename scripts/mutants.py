"""`just mutants <crate>`: a whole-crate cargo-mutants run, optionally split in two.

With `STUDIO_MUTANTS_REMOTE` set to an ssh host (e.g. `bonfire`) the run is sharded
round-robin: this machine runs `--shard 0/2`, the host runs `--shard 1/2`, and the
remote half's results merge into `mutants.out/remote/`. Unset or empty, the plain
local command runs.

The remote needs: git, a C toolchain, rustup and cargo-mutants at the local
version (the pinned toolchain installs itself on first use inside the tree).
"""

import io
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REMOTE_BASE = "~/studio-mutants"
REMOTE_REF = "refs/mutants/remote"


def ssh_binary() -> str:
    # Windows' own OpenSSH client shares the Windows ssh-agent that holds the
    # maintainer's key; Git Bash's /usr/bin/ssh comes first on PATH under bash
    # and fails with "Permission denied (publickey)".
    if os.name == "nt":
        candidate = Path(os.environ["SystemRoot"]) / "System32" / "OpenSSH" / "ssh.exe"
        if candidate.exists():
            return str(candidate)
    return "ssh"


def ssh(host: str, remote: str, **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(
        [ssh_binary(), "-o", "BatchMode=yes", host, remote], **kwargs
    )


def git(*args: str, **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=ROOT, **kwargs)


def snapshot() -> tuple[str, str]:
    """Commit the whole working tree (tracked edits and untracked non-ignored
    files: the run measures the tree under review) to `refs/mutants/remote`
    without touching the real index. Returns (tree, commit)."""
    index = git(
        "rev-parse", "--git-path", "mutants-remote.index",
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    index_path = Path(index)
    if not index_path.is_absolute():
        index_path = ROOT / index_path
    index_path.unlink(missing_ok=True)
    env = dict(os.environ, GIT_INDEX_FILE=str(index_path))
    # A fresh index re-adds every file, flooding stderr with CRLF warnings;
    # silence it on success only.
    added = subprocess.run(
        ["git", "add", "-A"], cwd=ROOT, env=env, capture_output=True, text=True
    )
    if added.returncode != 0:
        raise RuntimeError(f"git add -A failed:\n{added.stderr}")
    tree = git(
        "write-tree", env=env, check=True, capture_output=True, text=True
    ).stdout.strip()
    commit = git(
        "commit-tree", tree, "-p", "HEAD", "-m", "mutants remote snapshot",
        check=True, capture_output=True, text=True,
    ).stdout.strip()
    git("update-ref", REMOTE_REF, commit, check=True)
    return tree, commit


def remote_state(host: str) -> tuple[str | None, str | None, int, str]:
    """The remote clone's HEAD and tree (None without a clone), its CPU count and
    its cargo-mutants version, in one call. The binary is run directly (as
    `cargo-mutants mutants`), since the host has no default toolchain for `cargo`."""
    out = ssh(
        host,
        f"git -C {REMOTE_BASE}/tree rev-parse HEAD 'HEAD^{{tree}}' 2>/dev/null || true; "
        "nproc; ~/.cargo/bin/cargo-mutants mutants --version",
        check=True, capture_output=True, text=True,
    ).stdout.splitlines()
    head, tree = (out[0], out[1]) if len(out) == 4 else (None, None)
    return head, tree, int(out[-2]), out[-1].strip()


def transfer(host: str, remote_head: str | None) -> None:
    """Send the snapshot as a bundle and check it out remotely."""
    incremental = remote_head is not None and git(
        "cat-file", "-e", f"{remote_head}^{{commit}}"
    ).returncode == 0
    bundle = tempfile.NamedTemporaryFile(suffix=".bundle", delete=False)
    bundle.close()
    try:
        if incremental:
            git("bundle", "create", bundle.name, REMOTE_REF, f"^{remote_head}",
                check=True)
            print(f"sending incremental bundle (past remote HEAD {remote_head[:12]})",
                  flush=True)
        else:
            git("bundle", "create", bundle.name, REMOTE_REF, check=True)
            print("sending full bundle", flush=True)
        remote = (
            f"mkdir -p {REMOTE_BASE} && cd {REMOTE_BASE} && "
            "(test -d tree/.git || git init -q tree) && cat > snapshot.bundle && "
            "cd tree && git fetch -q ../snapshot.bundle refs/mutants/remote && "
            "git checkout -q --force --detach FETCH_HEAD && git clean -fdq"
        )
        with open(bundle.name, "rb") as data:
            ssh(host, remote, stdin=data, check=True)
    finally:
        Path(bundle.name).unlink(missing_ok=True)


def remote_config(host: str, nproc: int) -> None:
    """Write the remote's mutants.toml: the local file's `--jobs -9` is sized for
    the dev PC and would floor each remote cargo process at 1 job. Each of the
    two cargo processes gets half the remote CPUs."""
    config = tomllib.loads((ROOT / ".cargo/mutants.toml").read_text(encoding="utf-8"))
    half = max(1, nproc // 2)
    config["additional_cargo_args"] = ["--jobs", str(half)]
    config["additional_cargo_test_args"] = ["--", f"--test-threads={half}"]
    lines = []
    for key, value in config.items():
        serializable = isinstance(value, (str, int, bool)) or (
            isinstance(value, list) and all(isinstance(item, str) for item in value)
        )
        if not serializable:
            raise TypeError(f"cannot write mutants.toml key {key!r}: {value!r}")
        # JSON escapes are valid TOML basic-string escapes.
        lines.append(f"{key} = {json.dumps(value)}")
    ssh(host, f"cat > {REMOTE_BASE}/mutants.remote.toml",
        input="\n".join(lines) + "\n", text=True, check=True)


def local_mutants_version() -> str:
    return subprocess.run(
        ["cargo", "mutants", "--version"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.strip()


def run_split(
    crate: str, host: str, tree: str, remote_head: str | None, remote_tree: str | None,
    nproc: int,
) -> tuple[int, int, Path, float, float]:
    """The local shard starts at once, in the foreground with console output as
    today; a thread transfers the snapshot, writes the remote config and runs the
    remote shard, so the transfer counts against the remote side only. Returns
    (local, remote) codes, the remote console log's path and each side's seconds
    from the common start, so the summary shows which side waited."""
    log = tempfile.NamedTemporaryFile(mode="w+b", suffix=".log", delete=False)
    log.close()
    start = time.monotonic()
    remote: dict = {}

    def remote_half() -> None:
        try:
            if remote_tree == tree:
                print("remote tree up to date", flush=True)
            else:
                transfer(host, remote_head)
            remote_config(host, nproc)
            with open(log.name, "wb") as out:
                remote["code"] = subprocess.run(
                    [
                        ssh_binary(), "-o", "BatchMode=yes", host,
                        f"cd {REMOTE_BASE}/tree && . ~/.cargo/env && "
                        f"nice -n 19 ionice -c3 cargo mutants -p {crate} --jobs 2 "
                        "--shard 1/2 --sharding round-robin --config ../mutants.remote.toml",
                    ],
                    stdout=out, stderr=subprocess.STDOUT,
                ).returncode
        except (OSError, subprocess.CalledProcessError, TypeError) as error:
            # Re-raised in the main thread once the local half is done.
            remote["error"] = error
        remote["seconds"] = time.monotonic() - start

    thread = threading.Thread(target=remote_half)
    thread.start()
    local = subprocess.run(
        [
            "cargo", "mutants", "-p", crate, "--jobs", "2",
            "--shard", "0/2", "--sharding", "round-robin",
        ],
        cwd=ROOT,
    )
    local_seconds = time.monotonic() - start
    thread.join()
    if "error" in remote:
        raise RuntimeError(
            "the remote half failed before running; the local half covered shard 0/2 "
            "only (rerun, or unset STUDIO_MUTANTS_REMOTE for a local run)"
        ) from remote["error"]
    return (
        local.returncode, remote["code"], Path(log.name),
        local_seconds, remote["seconds"],
    )


def fetch_remote(host: str, log_path: Path) -> None:
    out = ROOT / "mutants.out" / "remote"
    fetched = ssh(
        host, f"tar -C {REMOTE_BASE}/tree -cf - mutants.out", capture_output=True
    )
    out.mkdir(parents=True, exist_ok=True)
    if fetched.returncode == 0 and fetched.stdout:
        with tarfile.open(fileobj=io.BytesIO(fetched.stdout), mode="r|") as tar:
            tar.extractall(out, filter="data")
    else:
        print("warning: remote produced no mutants.out", file=sys.stderr)
    (out / "remote_console.txt").write_bytes(log_path.read_bytes())
    log_path.unlink(missing_ok=True)


def count_lines(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines()) if path.exists() else 0


def summarize(
    local_code: int, remote_code: int, log_path: Path, seconds: tuple[float, float]
) -> int:
    local = ROOT / "mutants.out"
    remote = local / "remote" / "mutants.out"
    names = ["caught", "missed", "timeout", "unviable"]
    print(f"{'':8}{'caught':>7}{'missed':>7}{'timeout':>8}{'unviable':>9}{'seconds':>9}")
    totals = dict.fromkeys(names, 0)
    for (label, base), elapsed in zip((("local", local), ("remote", remote)), seconds):
        counts = {name: count_lines(base / f"{name}.txt") for name in names}
        for name in names:
            totals[name] += counts[name]
        print(
            f"{label:8}{counts['caught']:>7}{counts['missed']:>7}"
            f"{counts['timeout']:>8}{counts['unviable']:>9}{elapsed:>9.0f}"
        )
    print(
        f"{'total':8}{totals['caught']:>7}{totals['missed']:>7}"
        f"{totals['timeout']:>8}{totals['unviable']:>9}"
    )
    missed = (local / "missed.txt").read_text(encoding="utf-8") if (local / "missed.txt").exists() else ""
    if (remote / "missed.txt").exists():
        missed += (remote / "missed.txt").read_text(encoding="utf-8")
    (local / "missed_all.txt").write_text(missed, encoding="utf-8")
    if missed:
        print(missed, end="")
    if remote_code not in (0, 2, 3):
        # cargo-mutants: 2 = missed mutants, 3 = timeouts, 4 = baseline failed.
        print("--- remote log tail ---")
        print("\n".join(log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-20:]))
    return max(local_code, remote_code)


def main(argv: list[str]) -> int:
    crate = argv[1]
    host = os.environ.get("STUDIO_MUTANTS_REMOTE") or None
    if host is None:
        return subprocess.run(
            ["cargo", "mutants", "-p", crate, "--jobs", "2"], cwd=ROOT
        ).returncode

    # The snapshot and the one-call probe come first (a few seconds), so a
    # version mismatch stops before the local half starts.
    tree, _commit = snapshot()
    remote_head, remote_tree, nproc, remote_version = remote_state(host)
    local_version = local_mutants_version()
    if local_version != remote_version:
        print(
            f"cargo-mutants versions differ: local {local_version}, "
            f"remote {remote_version}; fix: cargo install --locked "
            f"cargo-mutants@{local_version.split()[-1]} on the remote"
        )
        return 1

    local_code, remote_code, log, local_seconds, remote_seconds = run_split(
        crate, host, tree, remote_head, remote_tree, nproc
    )
    fetch_remote(host, log)
    return summarize(
        local_code, remote_code,
        ROOT / "mutants.out" / "remote" / "remote_console.txt",
        (local_seconds, remote_seconds),
    )


if __name__ == "__main__":
    sys.exit(main(sys.argv))
