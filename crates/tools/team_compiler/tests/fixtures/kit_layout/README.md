# Kit layout fixtures

Both files are written by `scripts/provenance/kit_uv/kit_layout_fixture.py`, which reads the
uniform models of PES 2017 and PES 2021 (the maintainer's extracted game data) and nothing of
this crate: the golden must not come from the table it tests.

- `pre_fox_stripes.png` — a 1024 x 1024 kit drawn for the pre-Fox layout. Each sock island's
  rectangle (u 8 to 448, v 632 to 1160 in the units of a 2048-px texture; mirrored on the
  right) holds vertical stripes 32 units wide, each one flat color, different on the two socks.
  The rest is 64-unit tiles of flat colors, so a change outside the socks shows. Every color is
  exact in RGB565 and every edge sits on a 4-px block, so encoding the image as DXT1 loses
  nothing.
- `sock_stripes.txt` — one line per stripe lying wholly inside the island's seams (13 per
  sock): the island, the stripe's pre-Fox u range, its color, and the Fox u its centre lands on
  according to the models, all in 2048-px units. The centre is the mean of two medians, one per
  correspondence (the angle around the leg and the arc fraction of the ring; `pipeline.md`
  "Layout conversion"), each over every texel row of three sock variants. The last stripe of
  each sock (u 424 to 448) reaches past the seam at u 438, where the models have no points, and
  has no line.

`KIT_LAYOUT_REMAP` puts each stripe's centre within 3.8 units of its line; the tests allow 6.
The table is an approximation of a map that itself varies by about 10 units along the leg, so
the golden catches a wrong number, a wrong direction or a wrong mirror, not a difference of a
few texels.
