"""Append non-repetitive long rows to crates/libs/fox2/tests/fixtures/hash_golden.tsv.

The original golden's rows past 100 bytes are runs of `x`, so the 64-byte block loop of the
above-64 hash reads identical blocks whichever offset it uses and a wrong offset step passes.
These rows vary every byte (ASCII) or every character (non-ASCII), hashed by the reference.
Temp-and-replace; refuses to run twice (asserts the original 46 rows).
"""
import os
import sys
from pathlib import Path

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\pes-stadium-compiler-v1.0.0")
from Engines.stages.lib.fox2 import hash_string  # noqa: E402

GOLDEN = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\fox2\tests\fixtures\hash_golden.tsv")


def ascii_text(n: int) -> str:
    return "".join(chr(33 + (i * 37) % 94) for i in range(n))


def main() -> None:
    existing = GOLDEN.read_bytes().decode("utf-8")
    old_rows = existing.splitlines()
    assert len(old_rows) == 46, len(old_rows)
    assert existing.endswith("\n") and "\r" not in existing
    texts = [ascii_text(n) for n in (129, 150, 200, 257)]
    texts.append("".join(f"日本語{i:02}" for i in range(20)))  # 20 x (9 + 2) = 220 bytes
    for text in texts:
        assert "\t" not in text and "\n" not in text
        assert len(text.encode("utf-8")) > 128
    new_rows = "".join(f"{hash_string(t):012X}\t{t}\n" for t in texts)
    temporary = GOLDEN.with_suffix(".tsv.tmp")
    temporary.write_bytes((existing + new_rows).encode("utf-8"))
    os.replace(temporary, GOLDEN)
    print(f"appended {len(texts)} rows; {len(old_rows) + len(texts)} total")
    for t in texts:
        print(len(t.encode("utf-8")), f"{hash_string(t):012X}")


main()
