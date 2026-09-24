"""dds_convert converge fixtures (2.20d): the accepted formats the first set lacked, and two
hand-built edge cases, each with texconv's decode as the expected output.

Run after fixtures_dds.py (reads its source.png from the fixtures folder); does not touch the
first set's files.
"""
import hashlib
import shutil
import struct
import subprocess
from pathlib import Path

TEXCONV = r"C:\Data\4cc\4cc aet compiler\_old\4cc-aet-compiler\Engines\texconv.exe"
DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\dds_convert\tests\fixtures")
WORK = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\.tmp\dds_work_converge")
shutil.rmtree(WORK, ignore_errors=True)
WORK.mkdir(parents=True)
SRC = DEST / "source.png"


def run(*args):
    out = subprocess.run([TEXCONV, *args], capture_output=True, text=True, cwd=WORK)
    if out.returncode != 0:
        print(out.stdout, out.stderr)
        raise SystemExit("texconv failed")


def put(name, data):
    (DEST / name).write_bytes(data)
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


def decode(stem, path, extra=()):
    dec_dir = WORK / (stem + "_dec")
    dec_dir.mkdir()
    run("-nologo", "-y", "-f", "R8G8B8A8_UNORM", "-dx10", *extra, "-o", str(dec_dir), str(path))
    data = next(dec_dir.glob("*.dds")).read_bytes()
    put(f"{stem}.decoded.dds", data)
    return data


# Encoded by texconv from the shared test image, full chains.
for stem, fmt, extra in [
    ("bc2", "BC2_UNORM", ["-dx9"]),      # FourCC DXT3
    ("bc4", "BC4_UNORM", ["-dx10"]),     # DX10, DXGI 80 (texconv's default for BC4 is ATI1)
    ("ati1", "BC4_UNORM", ["-dx9"]),     # FourCC ATI1
    ("r8", "R8_UNORM", ["-dx10"]),       # DX10, DXGI 61
    ("l8_dx9", "R8_UNORM", ["-dx9"]),    # legacy luminance header (DDPF_LUMINANCE, 8 bits, r mask 0xff)
]:
    out_dir = WORK / stem
    out_dir.mkdir()
    run("-nologo", "-y", "-f", fmt, "-o", str(out_dir), *extra, str(SRC))
    produced = next(out_dir.glob("*.dds"))
    put(f"{stem}.dds", produced.read_bytes())
    decode(stem, produced)


def legacy_header(flags, height, width, pitch, mips, pf_flags, fourcc, bits, masks, caps1):
    return struct.pack("< 4s 7I 44x 2I 4s 5I 2I 12x", b"DDS ", 124, flags, height, width, pitch, 0,
                       mips, 32, pf_flags, fourcc, bits, *masks, caps1, 0)


# A 4x4 BC4 block whose endpoints are equal (a0 == a1 == 100): the six-value mode, where
# indices 6 and 7 are the constants 0 and 255. Indices 0..7 then 7..0 across the 16 texels.
indices = list(range(8)) + list(range(7, -1, -1))
packed = sum(index << (3 * texel) for texel, index in enumerate(indices))
block = bytes([100, 100]) + packed.to_bytes(6, "little")
equal = (struct.pack("< 4s 7I 44x 2I 4s 5I 2I 12x", b"DDS ", 124, 0x1 | 0x2 | 0x4 | 0x1000 | 0x80000,
                     4, 4, 8, 0, 1, 32, 0x4, b"DX10", 0, 0, 0, 0, 0, 0x1000, 0)
         + struct.pack("< 5I", 80, 3, 0, 1, 0) + block)
path = WORK / "bc4_equal_endpoints.dds"
path.write_bytes(equal)
put("bc4_equal_endpoints.dds", equal)
decode("bc4_equal_endpoints", path, ["-m", "1"])  # texconv would otherwise generate a chain

# A 6x2 24-bit BGR DDS with two mips whose rows are DWORD-padded: level 0 declares pitch 20
# (tight 18), level 1 is 3x1 (tight 9, stored in 12). texconv reads it with -dword.
rows0 = [bytes((y * 60 + x * 7 + c * 29) % 256 for x in range(6) for c in range(3)) + b"\xee\xee"
         for y in range(2)]
row1 = bytes((200 + x * 13 + c * 5) % 256 for x in range(3) for c in range(3)) + b"\xdd\xdd\xdd"
padded = (legacy_header(0x1 | 0x2 | 0x4 | 0x8 | 0x1000 | 0x20000, 2, 6, 20, 2, 0x40, b"\0\0\0\0", 24,
                        (0xff0000, 0xff00, 0xff, 0), 0x1000 | 0x8 | 0x400000)
          + b"".join(rows0) + row1)
path = WORK / "bgr24_dword_rows.dds"
path.write_bytes(padded)
put("bgr24_dword_rows.dds", padded)
decode("bgr24_dword_rows", path, ["-dword"])
# Shown for the record: texconv's byte-aligned default reads the same file differently.
alt_dir = WORK / "byte_aligned"
alt_dir.mkdir()
run("-nologo", "-y", "-f", "R8G8B8A8_UNORM", "-dx10", "-o", str(alt_dir), str(path))
alt = next(alt_dir.glob("*.dds")).read_bytes()
print("byte-aligned decode equals the -dword decode:", alt == (DEST / "bgr24_dword_rows.decoded.dds").read_bytes())
