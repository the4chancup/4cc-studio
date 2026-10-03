"""team_compiler texture fixtures (step 4.6): raster sources of the formats a member may export,
and a BC7 DDS with a full mip chain, for the compile tests of every accepted image format."""
import hashlib
import os
import shutil
import subprocess
from pathlib import Path

from PIL import Image

REPO = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio")
DEST = REPO / "crates" / "tools" / "team_compiler" / "tests" / "fixtures" / "textures"
BC7_SOURCE = REPO / "crates" / "libs" / "dds_convert" / "tests" / "fixtures" / "bc7.dds"


def gradient(width, height, alpha):
    """Red and green gradients, a blue checker with hard edges and, with `alpha`, an alpha ramp
    over the right half; deterministic, no noise."""
    pixels = bytearray()
    for y in range(height):
        for x in range(width):
            r = x * 255 // (width - 1)
            g = y * 255 // (height - 1)
            b = 255 if (x * 8 // width + y * 8 // height) % 2 == 0 else 32
            a = 255
            if alpha and x >= width // 2:
                a = (x - width // 2) * 255 // (width // 2 - 1)
            pixels += bytes((r, g, b, a))
    return Image.frombytes("RGBA", (width, height), bytes(pixels))


def put(name, write):
    """Writes `name` through a sibling temporary file, then moves it into place."""
    final = DEST / name
    temporary = DEST / (name + ".tmp")
    write(temporary)
    os.replace(temporary, final)
    data = final.read_bytes()
    print(f"{name}  {len(data)} bytes  sha256 {hashlib.sha256(data).hexdigest()[:16]}")


DEST.mkdir(parents=True, exist_ok=True)
skin = gradient(1024, 1024, alpha=True)
assert skin.getextrema()[3] == (0, 255), "the skin carries alpha"
put("skin.png", lambda path: skin.save(path, format="PNG"))
kit = gradient(256, 128, alpha=False)
assert kit.getextrema()[3] == (255, 255), "the kit is opaque"
put("kit.png", lambda path: kit.save(path, format="PNG"))
kit_back = gradient(128, 64, alpha=False)
put("kit_back.tga", lambda path: kit_back.save(path, format="TGA", rle=False))
put("bc7.dds", lambda path: shutil.copyfile(BC7_SOURCE, path))

# Step 4.6b: portraits in raster formats, the sizes the conversion refuses, and a codec
# `dds_convert` cannot decode.
portrait = gradient(128, 128, alpha=False)
put("portrait.png", lambda path: portrait.save(path, format="PNG"))
put("portrait.webp", lambda path: portrait.save(path, format="WEBP", lossless=True))
tiny = gradient(3, 3, alpha=False)
put("tiny.png", lambda path: tiny.save(path, format="PNG"))
odd = gradient(300, 300, alpha=False)
put("odd.png", lambda path: odd.save(path, format="PNG"))

TEXCONV = r"C:\Data\4cc\4cc aet compiler\_old\4cc-aet-compiler\Engines\texconv.exe"
WORK = REPO / ".tmp" / "tc_textures_work"
shutil.rmtree(WORK, ignore_errors=True)
WORK.mkdir(parents=True)
bc6h_source = WORK / "bc6h.png"
gradient(32, 16, alpha=False).save(bc6h_source, format="PNG")
completed = subprocess.run(
    [TEXCONV, "-f", "BC6H_UF16", "-y", "-o", str(WORK), str(bc6h_source)],
    capture_output=True,
    text=True,
)
assert completed.returncode == 0, completed.stdout + completed.stderr
put("bc6h.dds", lambda path: shutil.copyfile(WORK / "bc6h.DDS", path))
shutil.rmtree(WORK)
