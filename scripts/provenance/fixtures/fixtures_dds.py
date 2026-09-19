"""dds_convert fixtures: a small source image, texconv-encoded DDS variants, texconv-decoded references."""
import hashlib
import shutil
import subprocess
from pathlib import Path

from PIL import Image

TEXCONV = r"C:\Data\4cc\4cc aet compiler\_old\4cc-aet-compiler\Engines\texconv.exe"
DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\dds_convert\tests\fixtures")
WORK = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\dds_work")
shutil.rmtree(WORK, ignore_errors=True)
WORK.mkdir(parents=True)
DEST.mkdir(parents=True, exist_ok=True)

# 32x16: gradients, a hard edge, and an alpha ramp in the right half; deterministic, no noise.
w, h = 32, 16
img = Image.new("RGBA", (w, h))
for y in range(h):
    for x in range(w):
        r = x * 255 // (w - 1)
        g = y * 255 // (h - 1)
        b = 255 if (x // 8 + y // 8) % 2 == 0 else 32
        a = 255 if x < 16 else (x - 16) * 255 // 15
        img.putpixel((x, y), (r, g, b, a))
src = WORK / "source.png"
img.save(src)
# an opaque copy for BC1 (alpha would be lost)
opaque = img.copy()
opaque.putalpha(255)
src_opaque = WORK / "source_opaque.png"
opaque.save(src_opaque)


def run(*args):
    out = subprocess.run([TEXCONV, *args], capture_output=True, text=True, cwd=WORK)
    if out.returncode != 0:
        print(out.stdout, out.stderr)
        raise SystemExit("texconv failed")


def put(name, path):
    data = Path(path).read_bytes()
    (DEST / name).write_bytes(data)
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


# Encode: full mip chains, straight (non-sRGB) colors, no alpha premultiply.
variants = [
    ("bc1_opaque", "BC1_UNORM", src_opaque, ["-dx9"]),
    ("bc3", "BC3_UNORM", src, ["-dx9"]),
    ("bc7", "BC7_UNORM", src, []),
    ("bc5", "BC5_UNORM", src, []),
    ("ati2", "BC5_UNORM", src, ["-dx9"]),
    ("rgba8", "R8G8B8A8_UNORM", src, ["-dx10"]),
    ("bgra8_dx9", "B8G8R8A8_UNORM", src, ["-dx9"]),
]
for stem, fmt, source, extra in variants:
    out_dir = WORK / stem
    out_dir.mkdir()
    run("-nologo", "-y", "-f", fmt, "-o", str(out_dir), *extra, str(source))
    produced = next(out_dir.glob("*.dds"))
    put(f"{stem}.dds", produced)
    # Reference decode of the whole chain: texconv back to an uncompressed R8G8B8A8 DDS
    # (keeps every mip), which the tests read as raw pixels.
    dec_dir = WORK / (stem + "_dec")
    dec_dir.mkdir()
    run("-nologo", "-y", "-f", "R8G8B8A8_UNORM", "-dx10", "-o", str(dec_dir), str(produced))
    put(f"{stem}.decoded.dds", next(dec_dir.glob("*.dds")))

put("source.png", src)
put("source_opaque.png", src_opaque)
