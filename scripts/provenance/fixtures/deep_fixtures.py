"""Provenance of `crates/tools/team_compiler/tests/fixtures/deep/co Midcup Far.7z` (step 4.7a): a
solid LZMA2 `.7z` export of team `/co/` holding `Players/05 - Striker/boots.fmdl`, the bytes
of `boots_far.fmdl` beside it (written by `deep_far_vertex.rs`), and
`Players/05 - Striker/glove_l.fmdl`, the tracer export's, which has no far vertex: 7-Zip
makes no solid block of a single file. Its options are those of the `sources/` archives
(`7z a -t7z -ms=on -m0=lzma2`).

Run from the repository root. The archive is written beside its target and moved into place
once it has been listed and read back.
"""

import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DEEP = ROOT / "crates/tools/team_compiler/tests/fixtures/deep"
STAGE = ROOT / ".tmp/deep_fixture_stage"
SEVEN_ZIP = Path("C:/Program Files/7-Zip/7z.exe")
INNER = "Players/05 - Striker/boots.fmdl"
GLOVE = "Players/05 - Striker/glove_l.fmdl"
TRACER_GLOVE = (
    ROOT
    / "crates/tools/team_compiler/tests/fixtures/tracer/studio/egg Midcup Tracer"
    / "Players/05 - The Chad Stormworks Player/glove_l.fmdl"
)

assert SEVEN_ZIP.exists()
model = (DEEP / "boots_far.fmdl").read_bytes()
target = DEEP / "co Midcup Far.7z"
temp = DEEP / "co Midcup Far.7z.tmp"
assert not temp.exists(), temp

if STAGE.exists():
    shutil.rmtree(STAGE)
staged = STAGE / INNER
staged.parent.mkdir(parents=True)
staged.write_bytes(model)
(STAGE / GLOVE).write_bytes(TRACER_GLOVE.read_bytes())
subprocess.run(
    [str(SEVEN_ZIP), "a", "-t7z", "-ms=on", "-m0=lzma2", str(temp), "Players"],
    cwd=STAGE,
    check=True,
    capture_output=True,
)
listing = subprocess.run(
    [str(SEVEN_ZIP), "l", "-slt", "-t7z", str(temp)], check=True, capture_output=True, text=True
).stdout
assert "Solid = +" in listing, listing
assert "Path = " + INNER.replace("/", os.sep) in listing, listing
assert "Path = " + GLOVE.replace("/", os.sep) in listing, listing
read_back = subprocess.run(
    [str(SEVEN_ZIP), "e", "-so", "-t7z", str(temp), INNER.replace("/", os.sep)],
    check=True,
    capture_output=True,
).stdout
assert read_back == model
os.replace(temp, target)
shutil.rmtree(STAGE)
print(target.name, target.stat().st_size, "bytes")
