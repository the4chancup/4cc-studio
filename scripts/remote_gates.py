"""`just gates-remote`: `just gates` run on the mutants host (`STUDIO_MUTANTS_REMOTE`)
against the whole working tree, for while this machine should not carry the
load (2026-10-10: four MEMORY_MANAGEMENT bugchecks here, the last during a run;
`mutants.remote_only` moves the mutation runs the same way).

The tree goes over as a mutation run sends it (`mutants.snapshot`,
`mutants.transfer`), into the same remote clone, so this refuses to start while
a remote mutants half is running or uncollected. Its `target/` is gitignored and
survives the checkout, so a rerun builds incrementally. The gates run as the
transient service `GATES_UNIT`, memory-capped and CPU-idle like a mutants half
(the host's production Fluxer instance comes first), synchronously:
`systemd-run --wait --pipe` streams their output here and returns their exit
code. A dropped connection ends the run (the unit stops with its pipe); unlike a
mutation run, a rerun loses nothing.

The host runs the justfile's own `gates` recipe, with `just` installed in
`~/.cargo/bin` at this machine's version (`cargo install --locked just@<version>`,
as cargo-mutants is), so the two never parse the justfile differently.
"""

import sys

import mutants

GATES_UNIT = "studio-gates"
GATES_JOB = f"{mutants.REMOTE_BASE}/gates.sh"


def main() -> int:
    host = mutants.remote_host()
    if host is None:
        print("STUDIO_MUTANTS_REMOTE is not set")
        return 1
    state = mutants.remote_run_state(host)
    if state is None:
        print(f"cannot reach {host}")
        return 1
    if state != "none":
        print(
            f"a remote mutants half is {state} on {host}: the gates would rewrite its "
            "tree (`just mutants-collect` fetches a finished one)"
        )
        return 1
    tree, _commit = mutants.snapshot()
    remote_head, remote_tree, _nproc, _version = mutants.remote_state(host)
    if remote_tree == tree:
        print("remote tree up to date", flush=True)
    else:
        mutants.transfer(host, remote_head)
    job = (
        "set -e\n"
        ". ~/.cargo/env\n"
        # On disk, not the tmpfs `/tmp`, which counts against the unit's memory
        # (`mutants.REMOTE_TMPDIR`); the CLI tests write a sandbox export each.
        f'export TMPDIR="{mutants.REMOTE_TMPDIR}" && mkdir -p "$TMPDIR"\n'
        "just gates\n"
    )
    # A file, not bash's stdin, which a test reading stdin would eat lines of.
    mutants.ssh(
        host,
        f"cat > {GATES_JOB}",
        # Bytes, not text: a CRLF in the script would break bash on the host.
        input=job.encode("utf-8"),
        check=True,
    )
    print(f"running the gates on {host}", flush=True)
    run = mutants.ssh(
        host,
        f"sudo -n systemd-run --unit={GATES_UNIT} --uid=debian --wait --pipe --collect "
        f"-p MemoryMax={mutants.REMOTE_MEMORY_MAX} -p MemorySwapMax=0 -p CPUWeight=idle "
        f'--working-directory="$HOME/studio-mutants/tree" /bin/bash {GATES_JOB} '
        "< /dev/null",
    )
    print(f"gates on {host}: exit {run.returncode}")
    return run.returncode


if __name__ == "__main__":
    sys.exit(main())
