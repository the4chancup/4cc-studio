"""Append 33-64 byte non-ASCII rows to crates/libs/fox2/tests/fixtures/hash_golden.tsv.

The reference's `_hash_len33to64` leaves its second `a = fetch(16) + fetch(len - 32)` sum
unmasked before `_rotate(a, 37)`, so where that sum passes 2^64 (only possible when a fetched
word has its top bit set, i.e. non-ASCII text) its hash is not CityHash's: the C# tool and the
game wrap the sum. These rows are the reference's code with that one sum masked, and the script
asserts the unpatched reference disagrees on each, so the rows exercise exactly that case.
Temp-and-replace; refuses to run twice (asserts the 51 rows it expects to extend).
"""
import os
import sys
from pathlib import Path

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\pes-stadium-compiler-v1.0.0")
import Engines.stages.lib.fox2 as ref  # noqa: E402

GOLDEN = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\fox2\tests\fixtures\hash_golden.tsv")


def wrapping_len33to64(data: bytes) -> int:
    fetch = lambda at: ref._U64_LE.unpack_from(data, at)[0]  # noqa: E731
    u64, rotate, k0, k2 = ref._u64, ref._rotate, ref._K0, ref._K2
    length = len(data)
    z = fetch(24)
    a = u64(fetch(0) + u64((length + fetch(length - 16)) * k0))
    b = rotate(u64(a + z), 52)
    c = rotate(a, 37)
    a = u64(a + fetch(8))
    c = u64(c + rotate(a, 7))
    a = u64(a + fetch(16))
    vf = u64(a + z)
    vs = u64(b + rotate(a, 31) + c)
    a = u64(fetch(16) + fetch(length - 32))  # the reference omits this mask
    z = fetch(length - 8)
    b = rotate(u64(a + z), 52)
    c = rotate(a, 37)
    a = u64(a + fetch(length - 24))
    c = u64(c + rotate(a, 7))
    a = u64(a + fetch(length - 16))
    wf = u64(a + z)
    ws = u64(b + rotate(a, 31) + c)
    r = ref._shift_mix(u64((vf + ws) * k2 + (wf + vs) * k0))
    return u64(ref._shift_mix(u64(r * k0 + vs)) * k2)


def hash_with(len33to64, text: str) -> int:
    data = text.encode("utf-8") + b"\x00"
    assert 33 <= len(data) <= 64, len(data)
    seed1 = ref._u32((data[0] << 16) + (len(data) - 1))
    return ref._hash_len16(ref._u64(len33to64(data) - ref._SEED0), seed1) & 0xFFFFFFFFFFFF


def main() -> None:
    existing = GOLDEN.read_bytes().decode("utf-8")
    old_rows = existing.splitlines()
    assert len(old_rows) == 51, len(old_rows)
    assert existing.endswith("\n") and "\r" not in existing
    texts = ["é" * 20, "日本語" * 4]
    rows = []
    for text in texts:
        # The patched function equals the reference's own on ASCII, where the sum cannot wrap.
        assert hash_with(wrapping_len33to64, "a" * 40) == ref.hash_string("a" * 40)
        wrapped = hash_with(wrapping_len33to64, text)
        assert hash_with(ref._hash_len33to64, text) == ref.hash_string(text)
        assert wrapped != ref.hash_string(text), "row would not exercise the wrapping sum"
        rows.append(f"{wrapped:012X}\t{text}\n")
    temporary = GOLDEN.with_suffix(".tsv.tmp")
    temporary.write_bytes((existing + "".join(rows)).encode("utf-8"))
    os.replace(temporary, GOLDEN)
    sys.stdout.buffer.write("".join(rows).encode("utf-8"))
    print(f"{len(old_rows) + len(rows)} rows total")


main()
