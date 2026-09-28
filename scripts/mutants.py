"""`just mutants <crate>`: a whole-crate cargo-mutants run, optionally split in two.

With `STUDIO_MUTANTS_REMOTE` set to an ssh host (e.g. `bonfire`) the run is sharded
round-robin: this machine runs `--shard 0/2`, the host runs `--shard 1/2`, and the
remote half's results merge into `mutants.out/remote/`. Unset (in the process
and, on Windows, in the user's registry environment) or empty, the plain local
command runs.

The remote half runs detached on the host (`setsid nohup`, its state in
`~/studio-mutants/run/`), and this machine only polls it with short ssh calls,
so a dropped connection costs one poll, not the run: cargo-mutants cannot
resume, and a remote half tied to one long ssh session died with it (2.20i).
If this script itself stops, `just mutants-collect` waits for the remote half
and fetches it. A new run refuses to start while a remote half is running or
finished but not yet collected.

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
# The detached remote half's files: `pid`, `log`, `exit` ("<code> <seconds>",
# written when cargo-mutants returns) and `collected` (written by the fetch).
REMOTE_RUN = f"{REMOTE_BASE}/run"
POLL_SECONDS = 30
# Consecutive failed polls before giving up (an hour at 30 s); the remote half
# keeps running and `just mutants-collect` picks it up later.
MAX_POLL_FAILURES = 120
# Keepalives let a dead link fail a call in about a minute instead of hanging it.
SSH_OPTIONS = [
    "-o", "BatchMode=yes",
    "-o", "ConnectTimeout=20",
    "-o", "ServerAliveInterval=15",
    "-o", "ServerAliveCountMax=4",
]


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
    return subprocess.run([ssh_binary(), *SSH_OPTIONS, host, remote], **kwargs)


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


def remote_run_state(host: str) -> str | None:
    """`none` (no run, or the last one collected), `running`, `finished` (not yet
    collected) or `died` (a pid and no exit file: killed, or the host rebooted);
    None when the host could not be reached."""
    probe = ssh(
        host,
        f"cd {REMOTE_RUN} 2>/dev/null || {{ echo none; exit 0; }}; "
        "if [ -f collected ]; then echo none; "
        "elif [ -f exit ]; then echo finished; "
        "elif [ -f pid ] && kill -0 \"$(cat pid)\" 2>/dev/null; then echo running; "
        "elif [ -f pid ]; then echo died; "
        "else echo none; fi",
        capture_output=True, text=True,
    )
    state = probe.stdout.strip()
    if probe.returncode != 0 or state not in ("none", "running", "finished", "died"):
        return None
    return state


def launch_remote(host: str, crate: str) -> None:
    """Starts the remote half detached from this ssh session, which returns at
    once: its own session (`setsid`), no terminal, output to `run/log`, and
    `run/exit` written when cargo-mutants returns."""
    job = (
        "start=$(date +%s); . ~/.cargo/env && "
        f"nice -n 19 ionice -c3 cargo mutants -p {crate} --jobs 2 "
        "--shard 1/2 --sharding round-robin --config ../mutants.remote.toml; "
        "code=$?; echo \"$code $(( $(date +%s) - start ))\" > ../run/exit"
    )
    ssh(
        host,
        f"mkdir -p {REMOTE_RUN} && cd {REMOTE_RUN} && rm -f pid log exit collected && "
        f"cd {REMOTE_BASE}/tree && "
        f"(setsid nohup bash -c '{job}' > ../run/log 2>&1 < /dev/null & "
        "echo $! > ../run/pid)",
        check=True,
    )


def wait_remote(host: str) -> tuple[int, float]:
    """Polls until the remote half finishes; returns its exit code and its own
    seconds. A failed poll is retried; only MAX_POLL_FAILURES in a row give up,
    and the remote half is still collectable after that."""
    failures = 0
    announced = False
    while True:
        state = remote_run_state(host)
        if state is None:
            failures += 1
            print(f"remote poll failed ({failures} in a row), retrying", flush=True)
            if failures >= MAX_POLL_FAILURES:
                raise RuntimeError(
                    f"could not reach {host} for {failures} polls; the remote half keeps "
                    "running there: collect it with `just mutants-collect`"
                )
        elif state == "finished":
            exit_line = ssh(
                host, f"cat {REMOTE_RUN}/exit", capture_output=True, text=True, check=True
            ).stdout.split()
            return int(exit_line[0]), float(exit_line[1])
        elif state == "died":
            raise RuntimeError(
                "the remote half stopped without an exit code (killed, or the host "
                "rebooted): `just mutants-collect` fetches its partial results"
            )
        elif state == "none":
            raise RuntimeError("no remote half was started")
        else:
            failures = 0
            if not announced:
                print("waiting for the remote half", flush=True)
                announced = True
        time.sleep(POLL_SECONDS)


def run_split(
    crate: str, host: str, tree: str, remote_head: str | None, remote_tree: str | None,
    nproc: int,
) -> tuple[int, int, float, float]:
    """The local shard starts at once, in the foreground with console output as
    today; a thread transfers the snapshot, writes the remote config and launches
    the remote shard, so the transfer counts against the remote side only. Returns
    the (local, remote) codes and seconds: the local half's from the common
    start, the remote half's from its own launch."""
    start = time.monotonic()
    remote: dict = {}

    def remote_half() -> None:
        try:
            if remote_tree == tree:
                print("remote tree up to date", flush=True)
            else:
                transfer(host, remote_head)
            remote_config(host, nproc)
            launch_remote(host, crate)
            print("remote half launched", flush=True)
        except (OSError, subprocess.CalledProcessError, TypeError) as error:
            # Re-raised in the main thread once the local half is done.
            remote["error"] = error

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
    remote_code, remote_seconds = wait_remote(host)
    return local.returncode, remote_code, local_seconds, remote_seconds


def fetch_remote(host: str) -> None:
    """Copies the remote half's `mutants.out` and console log into
    `mutants.out/remote/`, then marks the run collected."""
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
    log = ssh(host, f"cat {REMOTE_RUN}/log 2>/dev/null || true", capture_output=True, check=True)
    (out / "remote_console.txt").write_bytes(log.stdout)
    ssh(host, f"touch {REMOTE_RUN}/collected", check=True)


def count_lines(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines()) if path.exists() else 0


def summarize(
    local_code: int, remote_code: int, log_path: Path,
    seconds: tuple[float | None, float | None],
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
        elapsed_text = "-" if elapsed is None else f"{elapsed:.0f}"
        print(
            f"{label:8}{counts['caught']:>7}{counts['missed']:>7}"
            f"{counts['timeout']:>8}{counts['unviable']:>9}{elapsed_text:>9}"
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


def collect(host: str) -> int:
    """`just mutants-collect`: waits for a remote half this script launched but
    did not fetch (it was stopped, or gave up polling), fetches it and prints the
    summary against whatever local `mutants.out` holds. A died half's partial
    results are fetched too, and the run exits 1."""
    state = remote_run_state(host)
    if state is None:
        print(f"cannot reach {host}")
        return 1
    if state == "none":
        print("no uncollected remote half")
        return 1
    if state == "died":
        print("the remote half died before it finished; fetching its partial results")
        remote_code, remote_seconds = 1, None
    else:
        remote_code, remote_seconds = wait_remote(host)
    fetch_remote(host)
    return summarize(
        0, remote_code,
        ROOT / "mutants.out" / "remote" / "remote_console.txt",
        (None, remote_seconds),
    )


def remote_host() -> str | None:
    """`STUDIO_MUTANTS_REMOTE` from the process, else (Windows) from the user's
    registry environment. A process started before the variable was set, such as
    an IDE's or an agent's shell, does not see it, and at 2.20i that silently
    turned a split run into a full local one. An empty value in the process
    opts out."""
    host = os.environ.get("STUDIO_MUTANTS_REMOTE")
    if host is None and os.name == "nt":
        import winreg

        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                host = winreg.QueryValueEx(key, "STUDIO_MUTANTS_REMOTE")[0]
        except OSError:
            host = None
    return host or None


def main(argv: list[str]) -> int:
    host = remote_host()
    if argv[1] == "--collect":
        if host is None:
            print("STUDIO_MUTANTS_REMOTE is not set")
            return 1
        return collect(host)
    crate = argv[1]
    if host is None:
        print("STUDIO_MUTANTS_REMOTE is not set: running every mutant on this machine", flush=True)
        return subprocess.run(
            ["cargo", "mutants", "-p", crate, "--jobs", "2"], cwd=ROOT
        ).returncode

    print(f"splitting the run with {host}", flush=True)
    # The remote must be free before the transfer rewrites its tree.
    state = remote_run_state(host)
    if state is None:
        print(f"cannot reach {host}")
        return 1
    if state != "none":
        status = {
            "running": "is still running",
            "finished": "has finished but is not collected",
            "died": "stopped before it finished",
        }[state]
        print(
            f"the remote half of an earlier run {status}: `just mutants-collect` "
            "fetches it, then start the new run"
        )
        return 1

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

    local_code, remote_code, local_seconds, remote_seconds = run_split(
        crate, host, tree, remote_head, remote_tree, nproc
    )
    fetch_remote(host)
    return summarize(
        local_code, remote_code,
        ROOT / "mutants.out" / "remote" / "remote_console.txt",
        (local_seconds, remote_seconds),
    )


if __name__ == "__main__":
    sys.exit(main(sys.argv))
