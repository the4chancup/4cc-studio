"""Writes crates/libs/fox2/tests/fixtures/double_golden.tsv: F64BITS<TAB>repr(value).

The reference decompiler writes a double property value as Python's repr(val), so repr is
the oracle. One row per case: fixed/exponent boundaries (-4, 16), integral values (".0"),
signed zero, denormals, extremes, non-finite.
"""
import math
import os
import struct
import sys

CASES = [
    0.0, -0.0, 1.0, -1.5, 0.5, 0.1, 100.0, 123.456, 1.0 / 3.0, 0.30000000000000004,
    1e15, 1234567890123456.0, 9999999999999998.0, 1e16, 1.2345678901234568e17, 1e22,
    1e-4, 0.00012345, 1e-5, 1.5e-5, -9.87654321e-7,
    5e-324, 2.2250738585072014e-308, 1.7976931348623157e308, -1e300,
    math.inf, -math.inf, math.nan,
    # An exact tie at 17 digits: repr rounds half to even (...312), a shortest-digits
    # formatter that rounds ties up prints ...313.
    1.00000762939453125,
    # A power of two: the gap below is half the gap above, so the nearest 16-digit text
    # (...062, ties to even) does not read back and repr takes the next one up (...063).
    2.0 ** -24,
]

def main() -> None:
    root = sys.argv[1]
    target = os.path.join(root, "crates", "libs", "fox2", "tests", "fixtures", "double_golden.tsv")
    rows = []
    for value in CASES:
        bits = struct.unpack("<Q", struct.pack("<d", value))[0]
        text = repr(value)
        assert "\t" not in text and "\n" not in text
        rows.append(f"{bits:016X}\t{text}\n")
    assert len(rows) == len(CASES) == 30
    temporary = target + ".tmp"
    with open(temporary, "wb") as handle:
        handle.write("".join(rows).encode("ascii"))
    os.replace(temporary, target)
    print(f"wrote {len(rows)} rows to {target}")

main()
