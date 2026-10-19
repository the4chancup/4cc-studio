"""Compile one tracer fixture's old export with Red (its checkout on branch main, clean), from a
copy of Red's folder, configured as the tracer READMEs say: settings.ini at its defaults but
pes_version, cpk_name = 4cc_90_tracer, move_cpks = 0, pause_allow = 0, updates_check = 0, and
pes_folder_path at a folder that does not exist.

usage: python run_red.py <tracer fixture folder name> <pes version>
  e.g. python run_red.py tracer 21; python run_red.py tracer_prefox 17

Writes .tmp/6_11/<tracer>/red_run/ (refuses to run when it exists) and Red's console output to
.tmp/6_11/<tracer>/red_run.log; the CPKs land in red_run/patches_output/.
"""

import shutil
import subprocess
import sys
from pathlib import Path

RED = Path(r"C:\Data\4cc\4cc aet compiler\4cc-aet-compiler-red")
WORKTREE = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio-wt-g")
FIXTURES = WORKTREE / "crates" / "tools" / "team_compiler" / "tests" / "fixtures"

tracer, version = sys.argv[1], sys.argv[2]
assert version in {"15", "16", "17", "18", "19", "20", "21"}, version
here = WORKTREE / ".tmp" / "6_11" / tracer
target = here / "red_run"
exports = sorted((FIXTURES / tracer / "old").iterdir())
assert len(exports) == 1 and exports[0].is_dir(), exports
export = exports[0]


def git(*args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(RED), *args], check=True, capture_output=True, text=True
    ).stdout.strip()


assert git("branch", "--show-current") == "main"
assert not git("status", "--porcelain", "--untracked-files=no"), "Red's checkout has changes"
# The fixture READMEs name this commit: a rerun after main moves is a new provenance.
assert git("rev-parse", "HEAD").startswith("1a48fa5"), git("rev-parse", "HEAD")
assert not target.exists(), f"{target} exists; remove it by hand first"
shutil.copytree(
    RED,
    target,
    ignore=shutil.ignore_patterns(
        ".git", "patches_output", "patches_contents", "exports_to_add", "extracted_teams",
        "extracted_referees", "temp", "*.log",
    ),
)

settings = target / "settings.ini"
text = settings.read_text(encoding="utf-8")
replacements = {
    "pes_version = 15": f"pes_version = {version}",
    "cpk_name = 4cc_90_test": "cpk_name = 4cc_90_tracer",
    "move_cpks = 1": "move_cpks = 0",
    r"pes_folder_path = C:\Program Files (x86)\Pro Evolution Soccer 20**": r"pes_folder_path = C:\nonexistent_pes_folder",
    "pause_allow = 1": "pause_allow = 0",
    "updates_check = 1": "updates_check = 0",
}
for old, new in replacements.items():
    assert text.count(old) == 1, old
    text = text.replace(old, new)
temporary = settings.with_suffix(".ini.new")
temporary.write_text(text, encoding="utf-8", newline="\r\n")
temporary.replace(settings)

shutil.copytree(export, target / "exports_to_add" / export.name)
with open(here / "red_run.log", "wb") as log:
    run = subprocess.run(
        [str(target / "Engines" / "embed" / "python.exe"), "Engines/compiler_main.py", "0", "--no-pause"],
        cwd=target,
        stdin=subprocess.DEVNULL,
        stdout=log,
        stderr=subprocess.STDOUT,
    )
assert run.returncode == 0, f"Red exited {run.returncode}; see {here / 'red_run.log'}"
print("output in", target / "patches_output")
