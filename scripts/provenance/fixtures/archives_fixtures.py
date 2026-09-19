"""Build the archives fixtures: a small export-like tree (real tiny PES files plus text), then
the same tree packed by 7-Zip as .7z (LZMA2 solid), .zip (Deflate), stored .zip, encrypted .7z
and .zip, and by PowerShell's Compress-Archive. New files only."""
import os
import shutil
import subprocess
from pathlib import Path

DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\archives\tests\fixtures")
SEVEN = r"C:\Program Files\7-Zip\7z.exe"
REFS = Path(r"C:\Data\4cc\Refs\26_4-summer_refs\exports_to_add\refs for Summer 26\Players")
WORK = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\archives_work")

assert not DEST.exists(), DEST
if WORK.exists():
    shutil.rmtree(WORK)
tree = WORK / "Sample Export"
(tree / "Faces" / "Player One").mkdir(parents=True)
(tree / "Faces" / "Player Two").mkdir(parents=True)
(tree / "Kits").mkdir()
(tree / "Empty").mkdir()
shutil.copy(REFS / "mudkip" / "Face" / "face.dds", tree / "Faces" / "Player One" / "face.dds")
shutil.copy(REFS / "spiderman" / "Face" / "materials.mtl", tree / "Faces" / "Player One" / "materials.mtl")
shutil.copy(REFS / "anon_suit" / "Face" / "face_diff.bin", tree / "Faces" / "Player Two" / "face_diff.bin")
(tree / "Note.txt").write_bytes(b"A sample export tree for the archives crate.\r\nTwo players, one kit note.\r\n")
(tree / "Kits" / "R\u00e9f.txt").write_bytes("Accents in a file name: R\u00e9f.\n".encode("utf-8"))
(tree / "Faces" / "Player Two" / "empty.txt").write_bytes(b"")

DEST.mkdir(parents=True)
shutil.copytree(tree, DEST / "sample")


def seven(args):
    subprocess.run([SEVEN, *args], cwd=WORK, check=True, capture_output=True)


seven(["a", "-t7z", "-mx=5", str(DEST / "sample.7z"), "Sample Export"])
seven(["a", "-tzip", "-mx=5", str(DEST / "sample_7zip.zip"), "Sample Export"])
seven(["a", "-tzip", "-mx=0", str(DEST / "sample_store.zip"), "Sample Export"])
seven(["a", "-t7z", "-pfoo", "-mhe=on", str(DEST / "encrypted.7z"), "Sample Export"])
seven(["a", "-tzip", "-pfoo", str(DEST / "encrypted.zip"), "Sample Export"])
subprocess.run(
    ["powershell", "-NoProfile", "-Command",
     f"Compress-Archive -LiteralPath '{tree}' -DestinationPath '{DEST / 'sample_ps.zip'}'"],
    check=True, capture_output=True,
)
for path in sorted(DEST.rglob("*")):
    if path.is_file():
        print(f"{path.stat().st_size:7}  {path.relative_to(DEST)}")
