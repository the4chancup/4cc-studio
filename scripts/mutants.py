"""`just mutants <crate>`: a whole-crate cargo-mutants run, optionally split in two.

With `STUDIO_MUTANTS_REMOTE` set to an ssh host (e.g. `bonfire`) the run is sharded
round-robin: this machine runs `--shard 0/2`, the host runs `--shard 1/2`, and the
remote half's results merge into `mutants.out/remote/`. Unset (in the process
and, on Windows, in the user's registry environment) or empty, the plain local
command runs.

The remote half runs as a transient systemd service on the host (`sudo -n
systemd-run --unit=studio-mutants`), detached from the ssh session by
construction, memory-capped and CPU-idle: the host also runs the production
Fluxer instance, an allocating mutant filled its RAM and swap at 2.20i, and
`nice` alone cannot keep the run off Fluxer's CPU (cgroup v2 weights ignore
it). Its state lives in `~/studio-mutants/run/` (`job.sh`, `pid`, `log`,
`exit`, `memory_peak`, `collected`), and this machine only polls it with
short ssh calls, so a dropped connection costs one poll, not the run:
cargo-mutants cannot resume, and a remote half tied to one long ssh session
died with it (2.20i). If this script itself stops, `just mutants-collect`
waits for the remote half and fetches it. A new run refuses to start while a
remote half is running or finished but not yet collected. A running half is
stopped by hand with `sudo systemctl stop studio-mutants` on the host.

The remote needs: git, a C toolchain, rustup and cargo-mutants at the local
version (the pinned toolchain installs itself on first use inside the tree).
"""

import io
import json
import os
import statistics
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
# The remote half's systemd unit: a transient service so its cgroup is
# memory-capped and CPU-idle — an allocating mutant OOM-killed the host at
# the 2.20i remainder run, and Fluxer's production services have priority.
REMOTE_UNIT = "studio-mutants"
# The unit's MemoryMax (with MemorySwapMax=0): at the cap the kernel
# OOM-kills inside this unit only, never in Fluxer's.
REMOTE_MEMORY_MAX = "6G"
# The detached remote half's files: `job.sh`, `pid` (the service's MainPID),
# `log`, `exit` ("<code> <seconds>", written when cargo-mutants returns),
# `memory_peak` (the unit cgroup's peak memory) and `collected` (written by
# the fetch).
REMOTE_RUN = f"{REMOTE_BASE}/run"
# What a run costs on this machine, measured per crate from the local half of
# every run: the mean seconds of one mutant and the unmutated baseline's
# seconds. `target/` is gitignored and survives the `mutants.out` wipes.
COST_CACHE = ROOT / "target" / "mutants-cost.json"
# The baseline (a cold build and test of the mutated packages in a fresh copy
# of the tree) of a crate no local run has measured yet: the 3.z run's
# 56 s + 8 s for team_compiler, studio_core and studio, rounded.
DEFAULT_BASELINE_SECONDS = 60.0
POLL_SECONDS = 30
# Below this estimated local wall time a diff run stays local: the split's
# fixed overhead (about a minute plus up to POLL_SECONDS of polling) eats
# the halving.
SPLIT_THRESHOLD_SECONDS = 240
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


def working_tree() -> str:
    """Write the whole working tree (tracked edits and untracked non-ignored
    files: a run measures the tree under review) as a git tree object, through
    a private index so the real one is not touched. Returns the tree's id."""
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
    return git(
        "write-tree", env=env, check=True, capture_output=True, text=True
    ).stdout.strip()


def snapshot() -> tuple[str, str]:
    """Commit the whole working tree (`working_tree`) to `refs/mutants/remote`.
    Returns (tree, commit)."""
    tree = working_tree()
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


def sized_config(cpus: int) -> str:
    """`.cargo/mutants.toml`'s text with each of the two cargo processes (`--jobs
    2`) given half of `cpus` logical CPUs, for build jobs and test threads
    alike, so together they use every CPU without oversubscribing it. The
    scripts run cargo-mutants at a low priority instead of holding CPUs back
    (`below_normal`), so the machine stays usable."""
    config = tomllib.loads((ROOT / ".cargo/mutants.toml").read_text(encoding="utf-8"))
    half = max(1, cpus // 2)
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
    return "\n".join(lines) + "\n"


def remote_config(host: str, nproc: int) -> None:
    """Write the remote's mutants.toml, sized for the remote's CPUs."""
    ssh(host, f"cat > {REMOTE_BASE}/mutants.remote.toml",
        input=sized_config(nproc), text=True, check=True)


def local_config() -> Path:
    """Write `target/mutants.local.toml`, sized for this machine's CPUs, and
    return its path for `--config`."""
    path = ROOT / "target" / "mutants.local.toml"
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix(".tmp")
    temp.write_text(sized_config(os.cpu_count() or 2), encoding="utf-8")
    os.replace(temp, path)
    return path


def below_normal() -> dict:
    """`subprocess` keyword arguments that start cargo-mutants at a below-normal
    priority, which every process it starts inherits (cargo, rustc, the
    linker, the test binaries): the run takes every CPU cycle the user's own
    work leaves idle, and yields the rest. On Windows the class is inherited
    by children by definition; on Linux the nice value is."""
    if os.name == "nt":
        return {"creationflags": subprocess.BELOW_NORMAL_PRIORITY_CLASS}
    return {"preexec_fn": lambda: os.nice(10)}


def local_mutants(args: list[str]) -> subprocess.CompletedProcess:
    """Runs `cargo mutants <args> --jobs 2` here, sized by `local_config` and at
    `below_normal` priority."""
    return subprocess.run(
        ["cargo", "mutants", *args, "--jobs", "2", "--config", str(local_config())],
        cwd=ROOT,
        **below_normal(),
    )


def killed_builds(out_dir: Path) -> list[str]:
    """The mutants under `out_dir` (a `mutants.out`) that cargo-mutants
    reported unviable because a build process was killed, not because the
    mutant does not compile. The remote half's memory cap kills a rustc that
    outgrows it, and cargo-mutants files the failed build as unviable, so the
    mutant goes untested without a miss or an error: at 3.z, 18 of the remote
    half's 19 mutants, eframe's dependency tree outgrowing 6 GiB."""
    path = out_dir / "outcomes.json"
    if not path.exists():
        return []
    killed = []
    for outcome in json.loads(path.read_text(encoding="utf-8"))["outcomes"]:
        if outcome["summary"] != "Unviable" or not outcome.get("log_path"):
            continue
        log = (out_dir / outcome["log_path"]).read_text(encoding="utf-8", errors="replace")
        if "SIGKILL" in log:
            killed.append(outcome["scenario"]["Mutant"]["name"])
    return killed


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


def launch_remote(host: str, selection: list[str]) -> None:
    """Writes `run/job.sh` (and `run/in.diff` for a `--in-diff` selection,
    the same diff the local half reads) and starts it as the `REMOTE_UNIT`
    transient service (`sudo -n systemd-run`), detached from this ssh session
    by construction: the unit is memory-capped (`MemoryMax`, no swap) and
    CPU-idle so Fluxer's production services on the host win, and
    `OOMPolicy=continue` keeps cargo-mutants alive past an in-unit OOM kill.
    `run/pid` is the service's MainPID; a failed `systemd-run` fails the
    launch (no fallback)."""
    if selection[0] == "--in-diff":
        remote_args = '--in-diff "$HOME/studio-mutants/run/in.diff"'
    else:
        remote_args = " ".join(selection)
    job = (
        "#!/bin/bash\n"
        "exec > ../run/log 2>&1\n"
        "start=$(date +%s)\n"
        f". ~/.cargo/env && nice -n 19 ionice -c3 cargo mutants {remote_args} --jobs 2 "
        "--shard 1/2 --sharding round-robin --config ../mutants.remote.toml\n"
        "code=$?\n"
        # `memory.peak` of this unit's cgroup tells whether the cap is tight;
        # `exit` stays the last file written: pollers treat it as finished.
        'peak="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)/memory.peak"\n'
        '[ -f "$peak" ] && cp "$peak" ../run/memory_peak\n'
        'echo "$code $(( $(date +%s) - start ))" > ../run/exit\n'
    )
    ssh(
        host,
        f"mkdir -p {REMOTE_RUN} && "
        f"rm -f {REMOTE_RUN}/pid {REMOTE_RUN}/log {REMOTE_RUN}/exit "
        f"{REMOTE_RUN}/collected {REMOTE_RUN}/memory_peak {REMOTE_RUN}/in.diff && "
        f"cat > {REMOTE_RUN}/job.sh",
        # Bytes, not text: `text=True` translates \n to \r\n on Windows and a
        # CRLF in the script makes bash and cargo-mutants fail.
        input=job.encode("utf-8"),
        check=True,
    )
    if selection[0] == "--in-diff":
        # Bytes again: the diff must stay LF for the remote cargo-mutants.
        ssh(
            host,
            f"cat > {REMOTE_RUN}/in.diff",
            input=Path(selection[1]).read_bytes(),
            check=True,
        )
    ssh(
        host,
        f"sudo -n systemd-run --unit={REMOTE_UNIT} --collect --uid=debian "
        f"-p MemoryMax={REMOTE_MEMORY_MAX} -p MemorySwapMax=0 -p OOMPolicy=continue "
        '-p CPUWeight=idle --working-directory="$HOME/studio-mutants/tree" '
        f'/bin/bash "$HOME/studio-mutants/run/job.sh" && '
        f"systemctl show -p MainPID --value {REMOTE_UNIT} > {REMOTE_RUN}/pid",
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
    selection: list[str], host: str, tree: str, remote_head: str | None,
    remote_tree: str | None, nproc: int,
) -> tuple[int, int, float, float]:
    """The local shard starts at once, in the foreground with console output as
    today; a thread transfers the snapshot, writes the remote config and launches
    the remote shard, so the transfer counts against the remote side only.
    `selection` is the mutant-selection arguments (`["-p", crate]` or
    `["--in-diff", path]`); the remote reads an uploaded copy of a diff.
    Returns the (local, remote) codes and seconds: the local half's from the
    common start, the remote half's from its own launch."""
    start = time.monotonic()
    remote: dict = {}

    def remote_half() -> None:
        try:
            if remote_tree == tree:
                print("remote tree up to date", flush=True)
            else:
                transfer(host, remote_head)
            remote_config(host, nproc)
            launch_remote(host, selection)
            print("remote half launched", flush=True)
        except (OSError, subprocess.CalledProcessError, TypeError) as error:
            # Re-raised in the main thread once the local half is done.
            remote["error"] = error

    thread = threading.Thread(target=remote_half)
    thread.start()
    local = local_mutants([*selection, "--shard", "0/2", "--sharding", "round-robin"])
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
    peak = ssh(
        host,
        f"cat {REMOTE_RUN}/memory_peak 2>/dev/null || true",
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    if peak:
        print(
            f"remote memory peak: {int(peak) / 2**30:.2f} GiB "
            f"(cap {REMOTE_MEMORY_MAX})"
        )
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
    return report_killed(max(local_code, remote_code))


def report_killed(code: int) -> int:
    """Lists the mutants whose build was killed (`killed_builds`), local half
    and remote half, and turns `code` into a failure when there are any: those
    mutants were never tested, and the counts above file them as unviable."""
    local = ROOT / "mutants.out"
    killed = killed_builds(local) + killed_builds(local / "remote" / "mutants.out")
    if not killed:
        return code
    print(
        f"{len(killed)} mutants were NOT tested: a build process was killed "
        "(the remote half's memory cap, or out of memory), and cargo-mutants "
        "counted them as unviable. Rerun them locally "
        "(STUDIO_MUTANTS_REMOTE= just mutants-diff <base>):"
    )
    for name in killed:
        print(f"  {name}")
    return max(code, 1)


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


def mutant_crate(file: str) -> str:
    """The crate a mutant lives in: the folder of the nearest `Cargo.toml` above
    its `file` field (`crates/libs/pes_savefile/src/...` -> `pes_savefile`,
    `crates/studio_core/src/...` -> `studio_core`). Counting path components
    instead named `studio` and `studio_core`, which sit directly under
    `crates/`, both `src`."""
    folder = (ROOT / file).parent
    while not (folder / "Cargo.toml").exists() and folder.parent != folder:
        folder = folder.parent
    return folder.name


def load_cost_cache() -> dict[str, dict[str, float]]:
    """`target/mutants-cost.json`: `mutant` (crate -> mean seconds per mutant)
    and `baseline` (crate -> the baseline seconds of the last local run that
    mutated it); empty tables when no run has written it yet. A cache in the
    older flat form (crate -> median seconds per mutant) seeds `mutant` with
    its values, minus the `src` key of the old crate naming: a median left
    out the timeouts and the cold builds, so it underestimates, but it is
    still nearer than the size fallback."""
    if not COST_CACHE.exists():
        return {"mutant": {}, "baseline": {}}
    cache = json.loads(COST_CACHE.read_text(encoding="utf-8"))
    if isinstance(cache.get("mutant"), dict) and isinstance(cache.get("baseline"), dict):
        return cache
    seeds = {crate: seconds for crate, seconds in cache.items() if crate != "src"}
    return {"mutant": seeds, "baseline": {}}


def cost_per_mutant(crate: str, cache: dict[str, dict[str, float]]) -> float:
    """A crate's measured mean seconds per mutant, or the size fallback when
    it has never run here: `1 + nonblank_lines / 1000` over
    `crates/**/<crate>/src/**/*.rs` (overestimates the mid-size crates, which
    errs toward splitting)."""
    if crate in cache["mutant"]:
        return cache["mutant"][crate]
    nonblank = 0
    for src in ROOT.glob(f"crates/**/{crate}/src/**/*.rs"):
        nonblank += sum(
            1
            for line in src.read_text(encoding="utf-8", errors="replace").splitlines()
            if line.strip()
        )
    return 1 + nonblank / 1000


def estimate_seconds(crates: list[str]) -> float:
    """The estimated wall seconds of running these mutants (one crate name per
    mutant) locally only: the unmutated baseline, which runs alone first,
    then every mutant's cost over the two jobs. The baseline is the largest
    measured for any of the crates: a cold build of the mutated packages in a
    fresh copy of the tree, a minute where a mutant takes seconds, so it
    dominates a small diff (3.z: 64 s of a 270 s run, while counting it as
    one mutant estimated the run at 80 s)."""
    cache = load_cost_cache()
    costs = [cost_per_mutant(crate, cache) for crate in crates]
    baseline = max(
        (cache["baseline"].get(crate, DEFAULT_BASELINE_SECONDS) for crate in set(crates)),
        default=0.0,
    )
    return baseline + sum(costs) / 2


def update_cost_cache() -> None:
    """Fold the local run's measured costs into `COST_CACHE`. Per crate, the
    mean of the summed phase durations over every mutant outcome, caught,
    missed, timed out or unviable alike, replacing the old value when the run
    had at least 5 of them (below that the mean is noise): each listed mutant
    is one of these, and the mean carries what a median dropped, a timeout's
    whole test timeout and each job's first, cold build (40-60 s at 3.z).
    The baseline's duration is recorded for every crate the run mutated.
    Only the local `mutants.out` counts; the remote half ran on other
    hardware."""
    path = ROOT / "mutants.out" / "outcomes.json"
    if not path.exists():
        return
    durations: dict[str, list[float]] = {}
    baseline = None
    for outcome in json.loads(path.read_text(encoding="utf-8"))["outcomes"]:
        scenario = outcome["scenario"]
        seconds = sum(phase["duration"] for phase in outcome["phase_results"])
        if isinstance(scenario, dict):
            durations.setdefault(mutant_crate(scenario["Mutant"]["file"]), []).append(seconds)
        elif scenario == "Baseline":
            baseline = seconds
    cache = load_cost_cache()
    for crate, values in durations.items():
        if len(values) >= 5:
            cache["mutant"][crate] = statistics.mean(values)
        if baseline is not None:
            cache["baseline"][crate] = baseline
    COST_CACHE.parent.mkdir(parents=True, exist_ok=True)
    temp = COST_CACHE.with_suffix(".tmp")
    temp.write_text(json.dumps(cache, indent=1) + "\n", encoding="utf-8")
    os.replace(temp, COST_CACHE)


def split(selection: list[str], host: str) -> int:
    """The whole split run for one selection: the busy-remote refusal, the
    snapshot, the version check, both halves, the fetch and the summary."""
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
        selection, host, tree, remote_head, remote_tree, nproc
    )
    update_cost_cache()
    fetch_remote(host)
    return summarize(
        local_code, remote_code,
        ROOT / "mutants.out" / "remote" / "remote_console.txt",
        (local_seconds, remote_seconds),
    )


def remote_host() -> str | None:
    """`STUDIO_MUTANTS_REMOTE` from the process, else (Windows) from the user's
    registry environment. A process started before the variable was set, such as
    an IDE's or an agent's shell, does not see it, and would silently run every
    mutant locally. An empty value in the process opts out."""
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
        code = local_mutants(["-p", crate]).returncode
        update_cost_cache()
        return report_killed(code)

    return split(["-p", crate], host)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
