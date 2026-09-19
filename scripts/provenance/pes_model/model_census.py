"""Layout census of .model fixtures: every byte range touched, gaps, unknown-field values.

Walks the pointer graph with struct only (no reference module), following the schema documented
in the reference parser. Output is evidence for the pes_model format/ design.
"""
import struct
import sys
import zlib
from pathlib import Path

FIXTURES = Path(r"C:\Data\4cc\Tools_Mine\4cc-studio\crates\libs\pes_model\tests\fixtures")


def unwrap(data):
    if len(data) >= 16 and data[4:8] == b"ESYS":
        return zlib.decompress(data[16:]), True
    return data, False


class Census:
    def __init__(self, data):
        self.data = data
        self.ranges = []  # (start, end, label)

    def take(self, start, size, label):
        if start + size > len(self.data):
            raise ValueError(f"{label}: {start}+{size} past end {len(self.data)}")
        self.ranges.append((start, start + size, label))
        return self.data[start:start + size]

    def cstring(self, start, label):
        end = self.data.index(b"\0", start)
        self.take(start, end - start + 1, label)
        return self.data[start:end].decode("utf-8")

    def record_array(self, base, label):
        """Returns (header_bytes, record_size, [(record_address, record_bytes)])."""
        offset, count, size = struct.unpack_from("<3I", self.take(base, 12, f"{label}.toc"), 0)
        header_size = offset - 12
        header = self.take(base + 12, header_size, f"{label}.hdr") if header_size else b""
        records = []
        cursor = base + offset
        for i in range(count):
            records.append((cursor, self.take(cursor, size, f"{label}[{i}]")))
            cursor += size
        return header, size, records

    def gaps(self):
        covered = sorted(self.ranges)
        out = []
        pos = 0
        for start, end, label in covered:
            if start > pos:
                out.append((pos, start, self.data[pos:start]))
            pos = max(pos, end)
        if pos < len(self.data):
            out.append((pos, len(self.data), self.data[pos:]))
        return out

    def overlaps(self):
        covered = sorted(self.ranges)
        out = []
        for (s1, e1, l1), (s2, e2, l2) in zip(covered, covered[1:]):
            if s2 < e1:
                out.append((l1, l2, s2, e1))
        return out


def census(path):
    raw = path.read_bytes()
    data, wrapped = unwrap(raw)
    c = Census(data)
    print(f"\n=== {path.name}: {len(raw)} bytes on disk, {len(data)} unwrapped, wesys={wrapped}")

    magic = c.take(0, 8, "magic")
    toc_offset, unk1, version, unk2, flags = struct.unpack("<IHHII", c.take(8, 16, "header"))
    print(f"magic={magic!r} toc_offset={toc_offset} unk1={unk1} version={version} unk2={unk2} flags={flags}")

    table_base = 8 + toc_offset
    _, _, section_records = c.record_array(table_base, "sections")
    section_bases = [table_base + struct.unpack("<I", r)[0] for _, r in section_records]
    print(f"section count={len(section_bases)}")
    order = sorted(range(len(section_bases)), key=lambda i: section_bases[i])
    print(f"file order of sections: {order}")
    for i, b in enumerate(section_bases):
        nxt = min([x for x in section_bases if x > b] + [len(data)])
        print(f"  section {i:2d} @ {b:6d} .. {nxt:6d} (len {nxt - b})")

    s = section_bases

    # Section 0: bone matrices + bone groups
    hdr, _, recs = c.record_array(s[0], "s0")
    print(f"s0: {len(recs)} entries, hdr={hdr.hex()}")
    for i, (_, r) in enumerate(recs):
        off = struct.unpack("<I", r)[0]
        base = s[0] + off
        dhdr, dsize, drecs = c.record_array(base, f"s0.e{i}")
        print(f"  entry {i} @ {base}: hdr={struct.unpack('<I', dhdr)} size={dsize} count={len(drecs)}")
        for j, (_, d) in enumerate(drecs):
            eoff, etype, ecount = struct.unpack("<3I", d)
            if etype == 7:
                c.take(base + eoff, 48, f"s0.e{i}.m{j}")
            elif etype == 1:
                ids = struct.unpack(f"<{ecount}H", c.take(base + eoff, 2 * ecount, f"s0.e{i}.g{j}"))
                print(f"    group {j}: type={etype} count={ecount} ids={ids} (blob @ {base + eoff})")
            else:
                print(f"    UNKNOWN entry type {etype} count {ecount}")
        if drecs and struct.unpack("<3I", drecs[0][1])[1] == 7:
            print(f"    {len(drecs)} matrices, first datum @ {base + struct.unpack('<3I', drecs[0][1])[0]}")

    # Section 5: bone names
    _, _, recs = c.record_array(s[5], "s5")
    names = [c.cstring(s[5] + struct.unpack("<I", r)[0], f"s5.n{i}") for i, (_, r) in enumerate(recs)]
    print(f"s5 bone names: {names}")

    # Section 6: materials
    _, _, recs = c.record_array(s[6], "s6")
    mats = {addr: c.cstring(s[6] + struct.unpack("<I", r)[0], f"s6.n{i}") for i, (addr, r) in enumerate(recs)}
    print(f"s6 materials: {mats}")

    # Section 2: annotation strings
    _, _, recs = c.record_array(s[2], "s2")
    annots = {}
    for i, (addr, r) in enumerate(recs):
        off, unk = struct.unpack("<II", r)
        annots[addr] = c.cstring(s[2] + off, f"s2.a{i}")
        print(f"s2 annotation @ {addr}: unk={unk} {annots[addr]!r}")
    if not recs:
        print("s2: empty")

    # Section 1: geometry
    ghdr, gsize, grecs = c.record_array(s[1], "s1")
    print(f"s1 geometry: {len(grecs)} records of {gsize}, hdr={ghdr.hex()}")
    geometries = {}
    for i, (addr, r) in enumerate(grecs):
        u1, vset_off, fdesc_off, u2 = struct.unpack_from("<4I", r, 0)
        misc_off = struct.unpack_from("<I", r, 16)[0] if gsize >= 20 else 0
        print(f"  geom {i} @ {addr}: unk1={u1} vset={vset_off} fdesc={fdesc_off} unk2={u2} misc={misc_off}")
        _, _, vs = c.record_array(s[1] + vset_off, f"s1.g{i}.vset")
        assert len(vs) == 1
        fields_off = struct.unpack("<I", vs[0][1])[0]
        fhdr, _, frecs = c.record_array(s[1] + fields_off, f"s1.g{i}.fields")
        u3, u4 = struct.unpack("<2I", fhdr)
        print(f"    fields @ {s[1] + fields_off}: hdr unk3={u3} unk4={u4}, {len(frecs)} fields")
        vcount = None
        for j, (_, f) in enumerate(frecs):
            foff, dtype, dfmt, fcount, u5 = struct.unpack("<5I", f)
            size = {3: 4, 4: 8, 5: 12, 6: 16, 8: 4, 9: 4}[dfmt]
            c.take(s[1] + foff, size * fcount, f"s1.g{i}.f{j}(type{dtype})")
            print(f"      field {j}: type={dtype} fmt={dfmt} count={fcount} unk5={u5} data @ {s[1] + foff} .. {s[1] + foff + size * fcount}")
            vcount = fcount
        _, _, fds = c.record_array(s[1] + fdesc_off, f"s1.g{i}.fdesc")
        assert len(fds) == 1
        fstart, u6, ffmt, fvcount, lod_levels, lod_off = struct.unpack("<6I", fds[0][1])
        print(f"    faces: start={s[1] + fstart} unk6={u6} fmt={ffmt} vcount={fvcount} lod_levels={lod_levels} lod_off={lod_off}")
        c.take(s[1] + fstart, 2 * fvcount, f"s1.g{i}.faces")
        if lod_levels:
            lods = struct.unpack(f"<{2 * lod_levels}I", c.take(s[1] + lod_off, 8 * lod_levels, f"s1.g{i}.lods"))
            print(f"    lods: {lods}")
        if misc_off:
            mbase = s[1] + misc_off
            _, _, ms = c.record_array(mbase, f"s1.g{i}.misc")
            moffs = [struct.unpack("<I", m)[0] for _, m in ms]
            print(f"    misc @ {mbase}: {len(ms)} entries offsets={moffs}")
            if len(moffs) >= 1:
                bb = struct.unpack("<8f", c.take(mbase + moffs[0], 32, f"s1.g{i}.bbox"))
                print(f"      bbox min={bb[0:4]} max={bb[4:8]}")
            if len(moffs) >= 2:
                mc = struct.unpack("<4I", c.take(mbase + moffs[1], 16, f"s1.g{i}.matcomb"))
                print(f"      matcomb={mc}")
            if len(moffs) >= 3:
                ro = struct.unpack("<I", c.take(mbase + moffs[2], 4, f"s1.g{i}.order"))
                print(f"      order={ro}")
            for k in range(3, len(moffs)):
                print(f"      EXTRA misc entry {k} offset {moffs[k]}")
        geometries[addr] = i

    # Section 4: meshes
    mhdr, msize, mrecs = c.record_array(s[4], "s4")
    print(f"s4 meshes: {len(mrecs)} records of {msize}, hdr={mhdr.hex()}")
    for i, (addr, r) in enumerate(mrecs):
        rel_geom, bg_off, ann_off, rel_mat, rel_s10 = struct.unpack_from("<iIIii", r, 0)
        unk_off = struct.unpack_from("<I", r, 20)[0] if msize >= 24 else 0
        geom_addr = s[4] + rel_geom
        mat_addr = s[4] + rel_mat
        print(f"  mesh {i} @ {addr}: geom->{geom_addr} (geometry #{geometries.get(geom_addr, '??')}) bg_off={bg_off} ann_off={ann_off} mat->{mat_addr} ({mats.get(mat_addr, '??')}) s10={rel_s10} unk_off={unk_off}")
        if bg_off:
            _, _, bgs = c.record_array(s[4] + bg_off, f"s4.m{i}.bg")
            for _, b in bgs:
                rel = struct.unpack("<i", b)[0]
                print(f"    bone group -> {s[4] + rel}")
        if ann_off:
            _, _, anns = c.record_array(s[4] + ann_off, f"s4.m{i}.ann")
            for _, a in anns:
                rel_str, s3_off, unk, atype = struct.unpack("<4i", a)
                print(f"    annotation -> {s[4] + rel_str} ({annots.get(s[4] + rel_str, '??')!r}) s3_off={s3_off} unk={unk} type={atype}")
        if unk_off:
            _, _, unks = c.record_array(s[4] + unk_off, f"s4.m{i}.unk")
            print(f"    unknownData: {len(unks)} entries (NOT walked)")

    # Section 7: model bbox + lod parameters
    _, _, recs = c.record_array(s[7], "s7")
    offs = [struct.unpack("<I", r)[0] for _, r in recs]
    print(f"s7: {len(recs)} entries offsets={offs}")
    if len(offs) >= 1:
        bb = struct.unpack("<8f", c.take(s[7] + offs[0], 32, "s7.bbox"))
        print(f"  bbox min={bb[0:4]} max={bb[4:8]}")
    if len(offs) >= 2:
        lod = struct.unpack("<Ifff", c.take(s[7] + offs[1], 16, "s7.lod"))
        print(f"  lod={lod}")
    for k in range(2, len(offs)):
        print(f"  EXTRA s7 entry {k} offset {offs[k]}")

    # Sections 3, 8, 9, 10: only the TOC
    for n in (3, 8, 9, 10):
        hdr, size, recs = c.record_array(s[n], f"s{n}")
        print(f"s{n}: hdr={hdr.hex()} size={size} count={len(recs)}")
        for j, (a, r) in enumerate(recs[:4]):
            print(f"   [{j}] @ {a}: {r.hex()}")

    print("-- gaps (unreferenced bytes):")
    for start, end, blob in c.gaps():
        kind = "zero" if not any(blob) else "DATA"
        print(f"  {start:6d} .. {end:6d} (len {end - start}) {kind} {blob[:32].hex()}")
    ov = c.overlaps()
    print(f"-- overlaps: {len(ov)}")
    for l1, l2, a, b in ov:
        print(f"  {l1} / {l2} : {a}..{b}")


for name in sys.argv[1:] or sorted(p.name for p in FIXTURES.glob("*.model")):
    census(FIXTURES / name)
