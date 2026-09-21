"""Build the color_tools template fixture: the PES 19 colored kit template sheet
(`Kit_col_template_pes19.png`, the community's UV layout reference with every zone painted a
different flat color) subsampled to 128x128 by nearest neighbour (no blending, so every pixel
is one of the sheet's zone colors) and stored as raw RGBA8, the form `extract_kit_colors`
takes. Then the expected answer, measured independently of the crate with PIL: the colors
inside the crate's shirt and shorts regions on the 128x128 image, counted per pixel. Prints the
literals the golden test asserts. New files only."""
from collections import Counter
from pathlib import Path

from PIL import Image

SRC = Path(r"C:/Data/4cc/Kits/Kit_col_template_pes19.png")
DEST = Path(r"C:/Data/4cc/Tools_Mine/4cc-studio/crates/libs/color_tools/tests/fixtures")
OUT = DEST / "kit_col_template_pes19_128.rgba"
SIZE = 128
# The crate's regions (color_tools.md "Dominant kit-color extraction", step 2).
SHIRT = (0.36, 0.05, 0.64, 0.88)
SHORTS = [(0.04, 0.61, 0.29, 0.90), (0.71, 0.61, 0.96, 0.90)]

assert not OUT.exists(), OUT
DEST.mkdir(parents=True, exist_ok=True)
small = Image.open(SRC).convert("RGBA").resize((SIZE, SIZE), Image.NEAREST)
raw = small.tobytes()
assert len(raw) == SIZE * SIZE * 4
OUT.write_bytes(raw)
print(f"{len(raw)} bytes -> {OUT.name}")


def zone_colors(regions):
    counts = Counter()
    for x0, y0, x1, y1 in regions:
        for y in range(int(y0 * SIZE), int(y1 * SIZE)):
            for x in range(int(x0 * SIZE), int(x1 * SIZE)):
                r, g, b, a = small.getpixel((x, y))
                if a >= 128:
                    counts[(r, g, b)] += 1
    total = sum(counts.values())
    return [(color, n / total) for color, n in counts.most_common(4)], total


for name, regions in [("shirt", [SHIRT]), ("shorts", SHORTS)]:
    top, total = zone_colors(regions)
    print(f"{name}: {total} opaque pixels")
    for color, share in top:
        print(f"  {color} {share:.3f}")
