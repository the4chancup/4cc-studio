"""Build the archives codec fixtures added at Phase 2 converge: the same `sample/` tree packed
by 7-Zip as a PPMd .7z (a codec the crate does not carry: a read error, not `Encrypted`), an
entry-encrypted .7z whose header is readable (lists, fails at `read`), and a bzip2 .zip (lists,
fails at `read` with the zip crate's own error). The tree is rebuilt from the committed
`sample/` folder, the empty `Empty/` folder re-created since git does not store it. New files
only: never overwrites."""
import shutil
import subprocess
from pathlib import Path

DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\archives\tests\fixtures")
SEVEN = r"C:\Program Files\7-Zip\7z.exe"
WORK = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\archives_work_codecs")

NEW = {
    "sample_ppmd.7z": ["-t7z", "-m0=PPMd"],
    "encrypted_names.7z": ["-t7z", "-pfoo"],
    "sample_bzip2.zip": ["-tzip", "-mm=BZip2"],
}

assert (DEST / "sample").is_dir(), DEST
for name in NEW:
    assert not (DEST / name).exists(), name
if WORK.exists():
    shutil.rmtree(WORK)
tree = WORK / "Sample Export"
shutil.copytree(DEST / "sample", tree)
(tree / "Empty").mkdir(exist_ok=True)

for name, flags in NEW.items():
    subprocess.run(
        [SEVEN, "a", *flags, str(DEST / name), "Sample Export"],
        cwd=WORK,
        check=True,
        capture_output=True,
    )
    print(f"{(DEST / name).stat().st_size:7}  {name}")
