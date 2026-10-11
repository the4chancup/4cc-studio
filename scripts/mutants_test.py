"""Tests of `mutants.py`'s readings of a finished run's files: which mutants never ran, which
builds were killed, and when a split run's summary fails. Each builds a `mutants.out` in a
temporary directory the way cargo-mutants writes one; nothing reaches the host. Run by `just
test` (`python scripts/mutants_test.py`)."""
import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutants  # noqa: E402


def mutant(name: str) -> dict:
    return {"Mutant": {"name": name}}


def write_run(out: Path, selected: int, outcomes: list[dict], logs: dict[str, str] = {}) -> None:
    """A `mutants.out` at `out`: a selection of `selected` mutants, the baseline and
    `outcomes`, and each `logs` entry as the log file of that name."""
    out.mkdir(parents=True, exist_ok=True)
    (out / "mutants.json").write_text(json.dumps([{}] * selected), encoding="utf-8")
    baseline = {"scenario": "Baseline", "summary": "Success", "phase_results": [{"duration": 1.0}]}
    (out / "outcomes.json").write_text(
        json.dumps({"outcomes": [baseline, *outcomes]}), encoding="utf-8"
    )
    for name, text in logs.items():
        (out / name).parent.mkdir(parents=True, exist_ok=True)
        (out / name).write_text(text, encoding="utf-8")


class NeverRan(unittest.TestCase):
    def test_a_half_stopped_in_its_baseline_ran_none_of_its_selection(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            (out / "mutants.json").write_text(json.dumps([{}, {}, {}]), encoding="utf-8")
            self.assertEqual(mutants.never_ran(out), (0, 3))

    def test_a_half_stopped_midway_ran_part_of_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            write_run(out, 3, [{"scenario": mutant("a"), "summary": "CaughtMutant"}])
            self.assertEqual(mutants.never_ran(out), (1, 3))

    def test_a_complete_half_and_one_with_no_selection_report_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            write_run(out, 1, [{"scenario": mutant("a"), "summary": "CaughtMutant"}])
            self.assertIsNone(mutants.never_ran(out))
            (out / "mutants.json").unlink()
            self.assertIsNone(mutants.never_ran(out))


class KilledBuilds(unittest.TestCase):
    def test_every_spelling_of_a_killed_build_and_a_signal_ended_cargo_count(self):
        logs = {
            "log/oom": "error: could not compile (signal: 9, SIGKILL: kill)",
            "log/linker": "collect2: fatal error: ld terminated with signal 9 [Killed]",
            "log/llvm": "LLVM ERROR: out of memory",
            "log/windows": "memory allocation of 1048576 bytes failed",
            "log/console": "process didn't exit successfully (exit code: 0xc0000142, STATUS_DLL_INIT_FAILED)",
            "log/mutant": "error[E0308]: mismatched types",
        }
        outcomes = [
            {"scenario": mutant(name), "summary": "Unviable", "log_path": f"log/{name}"}
            for name in ("oom", "linker", "llvm", "windows", "console", "mutant")
        ]
        outcomes.append({"scenario": mutant("signal"), "summary": "Failure"})
        outcomes.append({"scenario": mutant("caught"), "summary": "CaughtMutant"})
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            write_run(out, len(outcomes), outcomes, logs)
            self.assertEqual(
                mutants.killed_builds(out),
                ["oom", "linker", "llvm", "windows", "console", "signal"],
            )


class Summarize(unittest.TestCase):
    def summarize(self, root: Path, remote_only: bool = False) -> int:
        # The maintainer's switch is set here, not read: the test holds in either mode.
        saved = mutants.ROOT, mutants.REMOTE_ONLY
        mutants.ROOT, mutants.REMOTE_ONLY = root, remote_only
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                return mutants.summarize(0, 0, root / "remote_console.txt", (None, None))
        finally:
            mutants.ROOT, mutants.REMOTE_ONLY = saved

    def test_a_split_run_whose_local_half_never_started_fails(self):
        caught = [{"scenario": mutant("a"), "summary": "CaughtMutant"}]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_run(root / "mutants.out" / "remote" / "mutants.out", 1, caught)
            self.assertEqual(self.summarize(root), 1)
            # A remote-only run has no local half to start.
            self.assertEqual(self.summarize(root, remote_only=True), 0)
            write_run(root / "mutants.out", 1, caught)
            self.assertEqual(self.summarize(root), 0)


class CostCache(unittest.TestCase):
    def update(self, root: Path) -> bool:
        """Whether `update_cost_cache` over `root`'s `mutants.out` wrote the cache."""
        saved = mutants.ROOT, mutants.COST_CACHE
        mutants.ROOT, mutants.COST_CACHE = root, root / "cost.json"
        try:
            mutants.update_cost_cache()
            return mutants.COST_CACHE.exists()
        finally:
            mutants.ROOT, mutants.COST_CACHE = saved

    def test_a_run_with_a_killed_build_leaves_the_measured_costs(self):
        def outcomes(summary: str) -> list[dict]:
            return [
                {
                    "scenario": {"Mutant": {"name": f"m{i}", "file": "crates/libs/cpk/src/lib.rs"}},
                    "summary": summary,
                    "log_path": "log/m",
                    "phase_results": [{"duration": 1.0}],
                }
                for i in range(5)
            ]

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "mutants.out"
            killed = "process didn't exit successfully (exit code: 0xc0000142)"
            write_run(out, 5, outcomes("Unviable"), {"log/m": killed})
            self.assertFalse(self.update(Path(tmp)))
            write_run(out, 5, outcomes("CaughtMutant"), {"log/m": "test failed"})
            self.assertTrue(self.update(Path(tmp)))


if __name__ == "__main__":
    unittest.main()
