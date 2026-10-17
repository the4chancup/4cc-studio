"""Citation audit: does each test prove the acceptance scenario it cites? (CONTRIBUTING.md
"Citation audit").

`acceptance.py` checks that a cited ID names a live scenario; it cannot see a test that cites a
scenario stating another case. This asks SubQ Snap, a decision model with a 384K-token window,
of every (scenario, citing test) pair whether the test proves the scenario, and flags the pairs
under THRESHOLD. Scenarios and citations come from `acceptance.scan_repository()`, so both
scripts read them one way. The state is the scenario's text, the test (its citation lines
through the brace closing the function) and the functions of the test's own file it calls, one
level deep, since a test's work often sits in a helper. Measured on 2026-10-17 (DECISIONS.md):
the 732 citations at 7bd0bba against the same tests paired with a neighbouring scenario they
do not cite, AUC 0.995; 32 citations flagged, the three read in full all real.

Answers are cached in snap.out/cache.json by a hash of the model, question and state, so a
second run pays only for changed pairs. A flag is ruled like a reviewer concern and its row
goes into scripts/citation_rulings.md; the key hashes the state, so a ruled pair is not raised
again until its scenario, test or helpers change. The report is snap.out/citations.md.

The key: `SUBQ_API_KEY`, else the first line of the file `STUDIO_SNAP_KEY_FILE` names (read
from the user's registry environment too on Windows), outside the repository.

Usage: citation_audit.py
"""

import hashlib
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from datetime import date
from pathlib import Path

from acceptance import CITATION, ROOT, STARTS_WITH_ID, Scan, scan_repository

MODEL = "subq-snap-preview"
URL = "https://api.subq.ai/v1/systemone"
# Under it a pair is flagged. 0.3 flagged 32 of 732 citations and missed 18 of 732 swapped
# negatives; both real wrong citations of the project's history scored under it.
THRESHOLD = 0.3
OUT = ROOT / "snap.out"
RULINGS = ROOT / "scripts" / "citation_rulings.md"
QUESTION = {
    "proves": {
        "type": "noul",
        "instructions": (
            "The state holds an acceptance scenario of the project's specification (GIVEN, WHEN, "
            "THEN), one Rust test, and the helper functions the test calls from its own file. A "
            'scenario may list several cases ("or", "each", "then"); a test proves it when it '
            "sets up one of the GIVEN cases, does the WHEN, and asserts the THEN for that case. Is "
            "the behaviour this test checks the behaviour the scenario states?"
        ),
        "criteria": {
            "true": "the test proves the scenario, or one of the cases it lists",
            "false": "the test checks a behaviour or case the scenario does not state, or leaves its outcome unasserted",
        },
    }
}
FN = re.compile(r"^\s*(?:pub(?:\([\w:]+\))?\s+)?fn\s+(\w+)")
CALL = re.compile(r"\b([a-z_][a-z0-9_]*)\s*\(")
RULING_ROW = re.compile(r"^\| (?P<key>[0-9a-f]{16}) \| (?P<verdict>accepted|rejected) \|")


class AuditError(Exception):
    pass


@dataclass(frozen=True)
class Pair:
    scenario: str
    test: str
    state: str

    @property
    def key(self) -> str:
        return hashlib.sha256(self.state.encode("utf-8")).hexdigest()[:16]


def scenario_text(lines: list[str], start: int) -> str:
    """The scenario line at index `start` and its continuation lines."""
    body = [lines[start]]
    for line in lines[start + 1 :]:
        if not line.strip() or STARTS_WITH_ID.match(line) or line.startswith(("```", "#")):
            break
        body.append(line)
    return "\n".join(body)


def function_end(lines: list[str], start: int) -> int | None:
    """Index of the line whose brace closes the item opened at or after `start`."""
    depth, opened = 0, False
    for index in range(start, len(lines)):
        depth += lines[index].count("{") - lines[index].count("}")
        opened = opened or "{" in lines[index]
        if opened and depth <= 0:
            return index
    return None


def functions(lines: list[str]) -> dict[str, str]:
    """Every `fn` of a file, by name, with its source."""
    found = {}
    for index, line in enumerate(lines):
        if match := FN.match(line):
            end = function_end(lines, index)
            if end is not None:
                found.setdefault(match.group(1), "\n".join(lines[index : end + 1]))
    return found


def cited_test(lines: list[str], citation: int) -> tuple[str, str] | None:
    """(name, source) of the test the citation line at index `citation` stands above."""
    first = citation
    while first > 0 and CITATION.match(lines[first - 1]):
        first -= 1
    index = citation
    while index < len(lines) and CITATION.match(lines[index]):
        index += 1
    if index >= len(lines) or lines[index].strip() != "#[test]":
        return None
    end = function_end(lines, index)
    name = next((m.group(1) for line in lines[index : index + 4] if (m := FN.match(line))), None)
    if end is None or name is None:
        return None
    return name, "\n".join(lines[first : end + 1])


def state(scenario: str, path: str, test: str, helpers: list[str]) -> str:
    text = f"Acceptance scenario:\n\n{scenario}\n\nThe test ({path}):\n\n```rust\n{test}\n```"
    if helpers:
        text += "\n\nHelpers it calls, from the same file:\n\n```rust\n" + "\n\n".join(helpers) + "\n```"
    return text


def pairs(scan: Scan, read) -> list[Pair]:
    """Every live scenario's (scenario, citing test) pairs; `read(path)` gives a file's lines."""
    found = []
    for scenario_id, places in sorted(scan.citations.items()):
        scenario = scan.scenarios.get(scenario_id)
        if scenario is None or scenario.withdrawn:
            continue  # acceptance.py reports these
        plan, line = scenario.where.rsplit(":", 1)
        text = scenario_text(read(plan), int(line) - 1)
        for where in places:
            path, number = where.rsplit(":", 1)
            lines = read(path)
            test = cited_test(lines, int(number) - 1)
            if test is None:
                continue
            name, source = test
            own = functions(lines)
            called = [c for c in dict.fromkeys(CALL.findall(source)) if c in own and c != name]
            found.append(Pair(scenario_id, f"{path}::{name}", state(text, path, source, [own[c] for c in called])))
    return list({(p.scenario, p.test): p for p in found}.values())


def env(name: str) -> str | None:
    value = os.environ.get(name)
    if value is None and os.name == "nt":
        import winreg

        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                value = winreg.QueryValueEx(key, name)[0]
        except OSError:
            value = None
    return value or None


def api_key() -> str:
    if key := env("SUBQ_API_KEY"):
        return key
    if path := env("STUDIO_SNAP_KEY_FILE"):
        lines = [line.strip() for line in Path(path).read_text(encoding="utf-8").splitlines()]
        if found := next((line for line in lines if line and not line.startswith("#")), None):
            return found
    raise AuditError("no key: set SUBQ_API_KEY or STUDIO_SNAP_KEY_FILE")


def ask(key: str, text: str) -> float:
    body = json.dumps({"model": MODEL, "state": text, "questions": QUESTION}).encode("utf-8")
    for attempt in range(4):
        request = urllib.request.Request(
            URL, data=body, method="POST",
            headers={"Content-Type": "application/json", "Authorization": f"Bearer {key}"},
        )
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                return float(json.loads(response.read().decode("utf-8"))["answers"]["proves"]["noul"])
        except urllib.error.HTTPError as error:
            detail = error.read().decode("utf-8", "replace")[:300]
            if error.code in (429, 500, 502, 503) and attempt < 3:
                time.sleep(2 * (attempt + 1))
                continue
            raise AuditError(f"HTTP {error.code}: {detail}".replace(key, "<key>")) from None
        except urllib.error.URLError as error:
            if attempt < 3:
                time.sleep(2 * (attempt + 1))
                continue
            raise AuditError(f"network: {error.reason}") from None
    raise AuditError("unreachable")


def cache_key(text: str) -> str:
    return hashlib.sha256(json.dumps([MODEL, QUESTION, text], sort_keys=True).encode("utf-8")).hexdigest()


def ruled_keys(text: str) -> set[str]:
    return {m["key"] for line in text.splitlines() if (m := RULING_ROW.match(line))}


def report(scored: list[tuple[Pair, float]], ruled: set[str]) -> str:
    flags = sorted(((pair, score) for pair, score in scored if score < THRESHOLD), key=lambda item: item[1])
    new = [(pair, score) for pair, score in flags if pair.key not in ruled]
    lines = [
        "# Citation audit",
        "",
        f"{len(scored)} citations asked, {len(flags)} under P {THRESHOLD}: {len(new)} new, "
        f"{len(flags) - len(new)} already ruled (scripts/citation_rulings.md).",
        "",
    ]
    for pair, score in new:
        scenario, test = pair.state.split("\n\nThe test (", 1)
        lines += [f"## {pair.scenario} ← {pair.test} (P {score:.3f})", "", scenario.split("\n\n", 1)[1], "",
                  f"Row: `| {pair.key} | accepted/rejected | {date.today()} | {pair.scenario} | {pair.test} | {score:.2f} | reason |`", ""]
    return "\n".join(lines)


def main() -> int:
    scan = scan_repository()
    files: dict[str, list[str]] = {}

    def read(path: str) -> list[str]:
        if path not in files:
            files[path] = (ROOT / path).read_text(encoding="utf-8").splitlines()
        return files[path]

    found = pairs(scan, read)
    OUT.mkdir(exist_ok=True)
    cache_path = OUT / "cache.json"
    cache = json.loads(cache_path.read_text(encoding="utf-8")) if cache_path.exists() else {}
    todo = [pair for pair in found if cache_key(pair.state) not in cache]
    if todo:
        key = api_key()
        with ThreadPoolExecutor(max_workers=4) as pool:
            for pair, score in zip(todo, pool.map(lambda pair: ask(key, pair.state), todo)):
                cache[cache_key(pair.state)] = score
        temporary = cache_path.with_suffix(".tmp")
        temporary.write_text(json.dumps(cache, indent=0), encoding="utf-8")
        os.replace(temporary, cache_path)
    scored = [(pair, cache[cache_key(pair.state)]) for pair in found]
    ruled = ruled_keys(RULINGS.read_text(encoding="utf-8")) if RULINGS.exists() else set()
    text = report(scored, ruled)
    (OUT / "citations.md").write_text(text + "\n", encoding="utf-8")
    print(text.splitlines()[2])
    print(f"citation audit: {len(todo)} asked now, {len(found) - len(todo)} from the cache; report in snap.out/citations.md")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except AuditError as error:
        print(f"citation audit: {error}")
        sys.exit(1)
