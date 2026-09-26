"""dds_convert raster-format fixtures, sixth review round (2.20d): JPEG, BMP and WebP sources,
each with Pillow's decode (an independent reader) as the expected RGBA8. Writes only the files
named here; does not touch the earlier sets.

- `source_opaque.bmp`: `source_opaque.png` (32x16) saved by Pillow as a 24-bit BMP.
- `source.webp`: `source.png` (32x16, with alpha) saved by Pillow as lossless WebP with `exact`
  (RGB kept under zero alpha), so the decode must match the PNG's pixels exactly.
- `source_opaque.jpg`: `source_opaque.png` saved by Pillow as a baseline JPEG, quality 95, 4:4:4
  (no chroma subsampling, so no upsampling filter choice between decoders).
- `<name>.rgba`: Pillow's decode of each, converted to RGBA8, row-major, 32*16*4 bytes.
"""
import hashlib
from pathlib import Path

from PIL import Image

DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\dds_convert\tests\fixtures")


def put(name, data):
    (DEST / name).write_bytes(data)
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


opaque = Image.open(DEST / "source_opaque.png").convert("RGBA")
assert all(a == 255 for a in opaque.getchannel("A").getdata())
alpha = Image.open(DEST / "source.png").convert("RGBA")

opaque.convert("RGB").save(DEST / "source_opaque.bmp", format="BMP")
alpha.save(DEST / "source.webp", format="WEBP", lossless=True, exact=True)
opaque.convert("RGB").save(DEST / "source_opaque.jpg", format="JPEG", quality=95, subsampling=0)

for name in ("source_opaque.bmp", "source.webp", "source_opaque.jpg"):
    data = (DEST / name).read_bytes()
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")
    decoded = Image.open(DEST / name)
    decoded.load()
    rgba = decoded.convert("RGBA")
    assert rgba.size == (32, 16), rgba.size
    put(name + ".rgba", rgba.tobytes())

# Lossless formats must reproduce their PNG source exactly.
assert Image.open(DEST / "source_opaque.bmp").convert("RGBA").tobytes() == opaque.tobytes()
assert Image.open(DEST / "source.webp").convert("RGBA").tobytes() == alpha.tobytes()
jpeg = Image.open(DEST / "source_opaque.jpg").convert("RGBA").tobytes()
print("jpeg vs png max channel diff:", max(abs(a - b) for a, b in zip(jpeg, opaque.tobytes())))
