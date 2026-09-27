"""Writes crates/libs/fmdl/tests/fixtures/f16_golden.tsv: F32BITS<TAB>F16BITS<TAB>EXACT.

numpy's float32 -> float16 cast (round to nearest, ties to even) is the oracle for the
encoder; EXACT is 1 when the f32 input is itself a half value, so the row also checks the
decoder (float16 -> float32 is exact). NaN is left out: the conversion's NaN payload and
quiet-bit handling are tested by hand.

Rows: for every half exponent and a spread of mantissas, the half value, the exact midpoint to
the next half (a tie), one f32 ulp either side of the midpoint, and negated copies; plus the
overflow edge, the subnormal edge, f32 subnormals and zeros.
"""
import os
import sys

import numpy as np

MANTISSAS = [0, 1, 2, 3, 511, 512, 513, 1021, 1022, 1023]


def f32(value: float) -> np.float32:
    return np.float32(value)


def bits32(value: np.float32) -> int:
    return int(np.array([value], dtype=np.float32).view(np.uint32)[0])


def from_bits32(bits: int) -> np.float32:
    return np.array([bits], dtype=np.uint32).view(np.float32)[0]


def half_value(bits: int) -> np.float32:
    return np.array([bits], dtype=np.uint16).view(np.float16).astype(np.float32)[0]


def to_half_bits(value: np.float32) -> int:
    return int(np.array([value], dtype=np.float32).astype(np.float16).view(np.uint16)[0])


def is_half(value: np.float32) -> bool:
    return bits32(half_value(to_half_bits(value))) == bits32(value)


def main() -> None:
    root = sys.argv[1]
    target = os.path.join(root, "crates", "libs", "fmdl", "tests", "fixtures", "f16_golden.tsv")
    inputs: list[np.float32] = []
    for exponent in range(31):
        for mantissa in MANTISSAS:
            half = (exponent << 10) | mantissa
            value = half_value(half)
            if half == 0x7BFF:
                midpoint = f32(65520.0)
            else:
                midpoint = f32((np.float64(value) + np.float64(half_value(half + 1))) / 2)
            below = np.nextafter(midpoint, f32(-np.inf))
            above = np.nextafter(midpoint, f32(np.inf))
            inputs += [value, midpoint, below, above, -value, -midpoint]
    inputs += [
        f32(65504.0), f32(65519.996), f32(65536.0), f32(1e10), f32(np.inf), f32(-np.inf),
        f32(2.0 ** -25), np.nextafter(f32(2.0 ** -25), f32(1.0)), f32(1.5 * 2.0 ** -25),
        f32(2.0 ** -26), f32(1e-30), from_bits32(1), from_bits32(0x007FFFFF),
        from_bits32(0x00800000), f32(0.0), f32(-0.0),
    ]
    seen = set()
    rows = []
    for value in inputs:
        key = bits32(value)
        if key in seen:
            continue
        seen.add(key)
        assert not np.isnan(value)
        rows.append(f"{key:08X}\t{to_half_bits(value):04X}\t{1 if is_half(value) else 0}\n")
    temporary = target + ".tmp"
    with open(temporary, "w", newline="\n") as handle:
        handle.writelines(rows)
    os.replace(temporary, target)
    print(f"{len(rows)} rows -> {target}")


if __name__ == "__main__":
    main()
