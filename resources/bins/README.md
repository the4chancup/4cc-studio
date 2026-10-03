# Fallback bins

The bases the Team compiler builds a bin on when no installed CPK supplies one (a from-scratch
compile; Team compiler plan, `pipeline.md` "Bins accumulation"). They are embedded in the Studio
binary.

| File | Used for | Source | SHA-256 (first 12) |
|---|---|---|---|
| `UniformParameter18.bin` | PES 18 | Red's `Engines/bins/`, commit `c94e673` (2024-05-17, "Update fallback bin files") | `1f11016d8377` |
| `UniformParameter19.bin` | PES 19-21 | same | `12edb570d886` |
| `TeamColor.bin` | every version | same | `47db01932b31` |
| `UniColor.bin` | every version | same | `82d8e10f48e6` |

The per-version choice is Red's (19 and later build on the 19 base; one `TeamColor.bin` and one
`UniColor.bin` for every version), so a from-scratch compile's bins are comparable with Red's
output.

## Layout of the two color bins (measured 2026-10-03)

Both hold one fixed-size record per team, in team ID order starting at team 100, so team `id`'s
record starts at `(id - 100) * record size`. Both bases hold 821 records, teams 100 to 920.

- `TeamColor.bin`, 16-byte records: `u16` team ID, `u16` color count (4 in every record), then
  four colors of three bytes each (R, G, B).
- `UniColor.bin`, 85-byte records: `u32` team ID, `u8` kit count, then ten 8-byte kit entries:
  kit number (player kits count from 0, goalkeeper kits from 0x10; 0xFF marks an unused entry,
  the rest of which is zero), menu icon number, and two colors of three bytes each.

All integers are little-endian. The cup's own files in each install's `4cc_08_bins.cpk` have
the same layout (`scripts/provenance/fixtures/color_bins_compare.py`): PES 15, 16, 17 and 19
hold 821 records each, and PES 18's CPK holds neither file. The PES 21 install measured is
an old cup's, from before the database took its current form: its files hold 803 records
(teams 100 to 902), and its `TeamColor.bin` has one corrupt record (team 761: twelve color
bytes written over the header). Neither is a layout the compiler has to support; a current
PES 21 cup's files are still to be compared.
