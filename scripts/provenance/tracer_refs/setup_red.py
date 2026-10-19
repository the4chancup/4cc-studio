"""Copy Red (its checkout on branch main) to .tmp/6_10/red_run, configure it for the referee
tracer (PES 21, no deploy, default bins), and stage the export."""

import shutil
import subprocess
from pathlib import Path

RED = Path(r"C:\Data\4cc\4cc aet compiler\4cc-aet-compiler-red")
HERE = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio-wt-g\.tmp\6_10")
TARGET = HERE / "red_run"
EXPORT = HERE / "old" / "refs Tracer"

branch = subprocess.run(
    ["git", "-C", str(RED), "branch", "--show-current"], check=True, capture_output=True, text=True
).stdout.strip()
assert branch == "main", branch
dirty = subprocess.run(
    ["git", "-C", str(RED), "status", "--porcelain", "--untracked-files=no"],
    check=True,
    capture_output=True,
    text=True,
).stdout
assert not dirty, dirty
assert not TARGET.exists(), f"{TARGET} exists; remove it by hand first"
assert EXPORT.is_dir(), f"{EXPORT} missing"
shutil.copytree(
    RED,
    TARGET,
    ignore=shutil.ignore_patterns(
        ".git", "patches_output", "patches_contents", "exports_to_add", "extracted_teams",
        "extracted_referees", "temp", "*.log",
    ),
)

settings = TARGET / "settings.ini"
text = settings.read_text(encoding="utf-8")
replacements = {
    "pes_version = 15": "pes_version = 21",
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

exports = TARGET / "exports_to_add"
shutil.copytree(EXPORT, exports / EXPORT.name)
print("red ready at", TARGET)
