"""Writes the kit layout fixtures of team_compiler (step 4.10): a striped kit drawn for the
pre-Fox layout, and where the games' uniform models put each of its sock stripes in the Fox
layout. The second file is the golden KIT_LAYOUT_REMAP is tested against, so nothing here
reads the table: the centres come from the models alone (kit_uv_sock_angle.py's two
correspondences, every sock variant pooled, each island measured on its own).

- pre_fox_stripes.png: 1024 x 1024. Each sock island's rectangle (u 8 to 448, v 632 to 1160
  in 2048-px units, mirrored on the right) holds vertical stripes 32 units wide, each one flat
  colour; the rest is 64-unit tiles of flat colours. Every colour is exact in RGB565 and every
  edge sits on a 4-px block at 1024, so a DXT1 encode of the image is lossless.
- sock_stripes.txt: one line per stripe that lies wholly inside the island's seams: island,
  the stripe's pre-Fox u range, its colour, and the Fox u its centre lands on (2048-px units).

Also prints, per stripe, the two correspondences' centres and how far the table's map sits
from the golden centre, which is what the test's tolerance is taken from.
Usage: python kit_layout_fixture.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from PIL import Image
from kit_uv_sock_angle import TABLE, pooled_pairs

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..",
                   "crates", "tools", "team_compiler", "tests", "fixtures", "kit_layout")
R, SIZE = 2048, 1024
U0, U1, V0, V1 = 8, 448, 632, 1160  # the left sock island's rectangle, 2048-px units
STRIPE, TILE = 32, 64
SEAM = 438  # the island's far seam: a stripe reaching past it has no model points there


def rgb565(r5, g6, b5):
    return ((r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2))


PALETTE = [rgb565(r, g, b) for r in (2, 9, 16, 23, 30) for g in (6, 22, 38, 54) for b in (3, 15, 27)]


def stripe_colour(island, k):
    # 7 is coprime with the palette's 60 entries, so neighbours differ in two channels.
    return PALETTE[((k if island == "left" else 14 + k) * 7) % len(PALETTE)]


def main():
    by = pooled_pairs()
    stripes = [(U0 + STRIPE * k, min(U0 + STRIPE * (k + 1), U1)) for k in range((U1 - U0 + STRIPE - 1) // STRIPE)]
    lines, worst = [], 0.0
    xs, ys = zip(*TABLE)
    print("island  stripe      by angle  by arc   golden   table   table - golden")
    for island in ("left", "right"):
        for k, (a, b) in enumerate(stripes):
            if b > SEAM:
                continue
            centres = []
            for key in ("angle", "arc"):
                src, dst = by[(island, key)]
                centres.append(float(np.median(dst[(src >= a) & (src < b)])))
            golden = round(sum(centres) / 2, 1)
            table = float(np.interp((a + b) / 2, xs, ys))
            worst = max(worst, abs(table - golden))
            print(f"{island:5s}  {a:4d}..{b:4d}   {centres[0]:7.1f}  {centres[1]:7.1f}  {golden:7.1f}  {table:6.1f}   {table - golden:+5.1f}")
            r, g, bl = stripe_colour(island, k)
            if island == "left":
                lines.append(f"left  {a:4d} {b:4d}  {r:3d} {g:3d} {bl:3d}  {golden:6.1f}")
            else:
                lines.append(f"right {R - b:4d} {R - a:4d}  {r:3d} {g:3d} {bl:3d}  {R - golden:6.1f}")
    print(f"largest |table - golden|: {worst:.1f} px")

    scale = R // SIZE
    img = np.zeros((SIZE, SIZE, 3), np.uint8)
    for j in range(R // TILE):
        for i in range(R // TILE):
            colour = rgb565((i * 5 + j * 3) % 32, (i * 7 + j * 11) % 64, (i * 13 + j * 5) % 32)
            img[j * TILE // scale:(j + 1) * TILE // scale, i * TILE // scale:(i + 1) * TILE // scale] = colour
    for island in ("left", "right"):
        for k, (a, b) in enumerate(stripes):
            lo, hi = (a, b) if island == "left" else (R - b, R - a)
            img[V0 // scale:V1 // scale, lo // scale:hi // scale] = stripe_colour(island, k)

    os.makedirs(OUT, exist_ok=True)
    Image.fromarray(img, "RGB").save(os.path.join(OUT, "pre_fox_stripes.png"), optimize=True)
    header = [
        "# Where the games' uniform models put each sock stripe of pre_fox_stripes.png in the Fox",
        "# layout. Written by scripts/provenance/kit_uv/kit_layout_fixture.py; see README.md.",
        "# island, the stripe's pre-Fox u range, its colour (r g b), its centre's Fox u; 2048-px units.",
    ]
    with open(os.path.join(OUT, "sock_stripes.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(header + lines) + "\n")
    print(f"wrote {len(lines)} stripes and the {SIZE} x {SIZE} image to {os.path.normpath(OUT)}")


if __name__ == "__main__":
    main()
