"""dds_convert converge fixtures, second review round (2.20d): the NVTT v1 L8 header and TIFF
alpha association. Writes only the files named here; does not touch the earlier sets.

- `l8_nvtt1.dds`: a 4x4 8-bit DDS with DDPF_RGB and r mask 0xff (DirectXTex's DDSPF_L8_NVTT1,
  "NVTT v1 wrote these with RGB instead of LUMINANCE"), decoded by texconv 2024.1.1.1 into
  `l8_nvtt1.decoded.dds`.
- `rgba_associated.tiff` / `rgba_unassociated.tiff`: a 2x1 uncompressed RGBA8 TIFF whose
  ExtraSamples tag is 1 (associated alpha: stored color = straight color * alpha) or 2
  (unassociated). Same stored samples in both, so only the tag differs. Pillow, an independent
  reader, converts the first to straight (255, 0, 0, 128), (0, 0, 0, 0) and returns the second as
  stored, (128, 0, 0, 128), (0, 0, 0, 0) (printed below).
"""
import hashlib
import shutil
import struct
import subprocess
from pathlib import Path

TEXCONV = r"C:\Data\4cc\4cc aet compiler\_old\4cc-aet-compiler\Engines\texconv.exe"
DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\dds_convert\tests\fixtures")
WORK = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\dds_work_converge_r2")
shutil.rmtree(WORK, ignore_errors=True)
WORK.mkdir(parents=True)


def put(name, data):
    (DEST / name).write_bytes(data)
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


# --- NVTT v1 L8 ---
pixels = bytes((17 * i + 3) % 256 for i in range(16))
header = struct.pack("< 4s 7I 44x 2I 4s 5I 2I 12x", b"DDS ", 124, 0x1 | 0x2 | 0x4 | 0x8 | 0x1000,
                     4, 4, 4, 0, 1, 32, 0x40, b"\0\0\0\0", 8, 0xff, 0, 0, 0, 0x1000, 0)
nvtt = header + pixels
src = WORK / "l8_nvtt1.dds"
src.write_bytes(nvtt)
put("l8_nvtt1.dds", nvtt)
dec_dir = WORK / "dec"
dec_dir.mkdir()
out = subprocess.run([TEXCONV, "-nologo", "-y", "-f", "R8G8B8A8_UNORM", "-dx10", "-m", "1",
                      "-o", str(dec_dir), str(src)], capture_output=True, text=True)
if out.returncode != 0:
    print(out.stdout, out.stderr)
    raise SystemExit("texconv failed")
decoded = next(dec_dir.glob("*.dds")).read_bytes()
put("l8_nvtt1.decoded.dds", decoded)
texels = decoded[148:148 + 64]
grey = all(texels[4 * i] == texels[4 * i + 1] == texels[4 * i + 2] == pixels[i] and texels[4 * i + 3] == 255
           for i in range(16))
print("texconv decodes NVTT1 L8 to grey (R = G = B = L, A = 255):", grey)


# --- TIFF alpha association ---
def tiff(extra_samples):
    # Stored samples: pixel 0 is straight (255, 0, 0) at alpha 128 premultiplied to
    # (128, 0, 0, 128); pixel 1 is fully transparent (0, 0, 0, 0).
    data = bytes([128, 0, 0, 128, 0, 0, 0, 0])
    entries = []  # (tag, type, count, value-or-offset)
    # Layout: 8-byte header, pixel data at 8, BitsPerSample array at 16, IFD at 24.
    bps_offset = 16
    ifd_offset = 24
    entries = [
        (256, 3, 1, 2),             # ImageWidth
        (257, 3, 1, 1),             # ImageLength
        (258, 3, 4, bps_offset),    # BitsPerSample 8,8,8,8
        (259, 3, 1, 1),             # Compression: none
        (262, 3, 1, 2),             # PhotometricInterpretation: RGB
        (273, 4, 1, 8),             # StripOffsets
        (277, 3, 1, 4),             # SamplesPerPixel
        (278, 3, 1, 1),             # RowsPerStrip
        (279, 4, 1, len(data)),     # StripByteCounts
        (284, 3, 1, 1),             # PlanarConfiguration: chunky
        (338, 3, 1, extra_samples), # ExtraSamples
    ]
    out = bytearray(struct.pack("<2sHI", b"II", 42, ifd_offset))
    out += data
    out += struct.pack("<4H", 8, 8, 8, 8)
    assert len(out) == ifd_offset
    out += struct.pack("<H", len(entries))
    for tag, typ, count, value in entries:
        if typ == 3 and count == 1:
            out += struct.pack("<HHIHH", tag, typ, count, value, 0)
        else:
            out += struct.pack("<HHII", tag, typ, count, value)
    out += struct.pack("<I", 0)
    return bytes(out)


for name, extra in [("rgba_associated.tiff", 1), ("rgba_unassociated.tiff", 2)]:
    put(name, tiff(extra))

from PIL import Image  # noqa: E402

for name in ["rgba_associated.tiff", "rgba_unassociated.tiff"]:
    with Image.open(DEST / name) as image:
        print(name, "Pillow mode:", image.mode, "RGBA:", list(image.convert("RGBA").getdata()))
