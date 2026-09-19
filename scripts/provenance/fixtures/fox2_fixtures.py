"""Write the fox2 fixtures into crates/libs/fox2/tests/fixtures/ (new files only, temp-and-replace):
four Konami .fox2 binaries, reference XML decompiles (no dictionary; plus one with a dictionary),
reference-compiled binaries with the writer's slack stripped, a hash golden and a float golden."""
import os
import struct
import sys
from pathlib import Path

sys.path.insert(0, r"C:\Data\4cc\Tools_4cc\pes-stadium-compiler-v1.0.0")
from Engines.stages.lib.fox2 import (  # noqa: E402
    compile_fox2_binary,
    decompile_fox2,
    float_to_str,
    hash_string,
    load_dictionary,
)
from Engines.stages.lib.fpk import FpkFile  # noqa: E402

DEST = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\fox2\tests\fixtures")
DEST.mkdir(parents=True, exist_ok=True)
DICT_PATH = Path(r"C:\Data\4cc\Tools_4cc\pes-stadium-compiler-v1.0.0\Engines\stages\lib\fox_dictionary.txt")
DICT = load_dictionary(DICT_PATH)


def put(name, data: bytes):
    path = DEST / name
    assert not path.exists(), path
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_bytes(data)
    os.replace(tmp, path)
    print(f"{len(data):8}  {name}")


def strip_slack(compiled: bytes) -> bytes:
    """The reference writer returns its over-allocated buffer; cut at the aligned `end` trailer."""
    at = struct.unpack_from("<i", compiled, 12)[0]
    while True:
        h = struct.unpack_from("<Q", compiled, at)[0]
        at += 8
        if h == 0:
            break
        n = struct.unpack_from("<I", compiled, at)[0]
        at += 4 + n
    at = (at + 15) & ~15
    assert compiled[at:at + 5] == b"\x00\x00end", compiled[at:at + 5]
    at = (at + 5 + 15) & ~15
    assert not any(compiled[at:]), "non-zero bytes past the trailer"
    return compiled[:at]


SOURCES = [
    ("audi_low_parts", r"C:\Data\4cc\Tools_Mine\4cc refs compiler\dt00_x64_files\Asset\model\bg\common\audi\#Win\audiLowParts_model.fpkd",
     "/Assets/pes16/model/bg/common/audi/audiLowParts_model.fox2"),
    ("boots_edit_k0051", r"C:\Data\4cc\Models\Model18\FPKs\boots_edit.fpkd",
     "/Assets/pes16/model/character/boots/k0051/boots_edit.fox2"),
    ("steward_sit_st074", r"C:\Data\4cc\Teams_Main\JP\jp_staff_b1\Asset\model\bg\st074\staff\#Win\staff_st074.fpkd",
     "/Assets/pes16/model/bg/st074/staff/st074_st2019_steward_sit.fox2"),
    ("edit_spike", r"C:\Data\4cc\Tools_Mine\4cc refs compiler\dt00_x64_files\Asset\model\light\#Win\EditSpike.fpkd",
     "/Assets/pes16/model/light/EditSpike.fox2"),
]

for stem, fpkd, entry in SOURCES:
    fpk = FpkFile()
    fpk.read(Path(fpkd).read_bytes())
    data = fpk.entries[entry]
    put(f"{stem}.fox2", data)
    xml = decompile_fox2(data, None)
    put(f"{stem}.fox2.xml", xml.encode("utf-8"))
    compiled = strip_slack(compile_fox2_binary(xml))
    # The entity region must be identical to Konami's; only the string table order differs.
    table_at = struct.unpack_from("<i", data, 12)[0]
    assert compiled[:table_at] == data[:table_at]
    put(f"{stem}.compiled.fox2", compiled)
    if stem == "audi_low_parts":
        with_dict = decompile_fox2(data, DICT)
        assert with_dict != xml
        put(f"{stem}.dict.fox2.xml", with_dict.encode("utf-8"))
        # The dictionary lines the resolution needs, for the test's own small dictionary.
        import re
        resolved = set(re.findall(r"<value>([^<]*)</value>", with_dict)) - set(re.findall(r"<value>([^<]*)</value>", xml))
        print("  resolved via dictionary:", sorted(resolved))

# Hash golden: one dictionary line per length across every CityHash length class, plus edge strings.
lines = DICT_PATH.read_text(encoding="utf-8-sig").splitlines()
by_len = {}
for line in lines:
    by_len.setdefault(len(line.encode("utf-8")), line)
wanted = [0, 1, 2, 3, 4, 5, 7, 8, 9, 11, 12, 15, 16, 17, 20, 23, 24, 31, 32, 33, 40, 47, 48, 55, 56, 63, 64, 65, 72, 80, 95, 96, 100, 127, 128, 129, 150, 200]
rows = []
for n in wanted:
    text = "" if n == 0 else by_len.get(n)
    if text is None:
        text = "x" * n
    rows.append(text)
rows += ["DataSet", "name", "dataList", "TransformEntity", "StadiumModel0000", "/Assets/pes16/model/bg/common/audi/scenes/au00.skl", "é", "日本語"]
golden = "".join(f"{hash_string(t):012X}\t{t}\n" for t in rows)
put("hash_golden.tsv", golden.encode("utf-8"))

# Float golden: f32 bit pattern (hex) -> the reference's text, covering every formatting branch.
floats = [0.0, -0.0, 1.0, -1.0, 0.5, 1.2, 0.2, 0.8, 0.05, 45.0, 4000.0, 123456.0, 1234567.0, 12345678.0, 123456789.0,
          0.0001, 0.00001, 0.000123, 1e10, 1e-10, 3.4028235e38, 1.17549435e-38, 4.036323e-39, 1.401298e-45,
          3.1415927, 2.5e-5, 100000.0, 1000000.0, 10000000.0, 0.1, 0.3, 1.5, -25.0, 0.30000001192092896, 16777216.0, 16777217.0]
rows = []
for value in floats:
    bits = struct.unpack("<I", struct.pack("<f", value))[0]
    rows.append(f"{bits:08X}\t{float_to_str(struct.unpack('<f', struct.pack('<f', value))[0])}\n")
put("float_golden.tsv", "".join(rows).encode("utf-8"))
