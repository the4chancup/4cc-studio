#!/usr/bin/env python3
"""PES Fox skeleton file (.skl) format — reference parser.

Written as a style example for the 4cc Studio rewrite. The format was
reverse-engineered from the template body.skl; the pes-fmdl Blender addon's
PesSkeletonData.py hardcoded its bone positions and parent hierarchy from
these files once but never parsed them at runtime.

Format layout (all little-endian):

    offset  size  field
    0       4     magic / version (always 12)
    4       4     bone count (N)
    8       4     record size (always 56)
    12      56*N  bone records
    ...           name table (concatenated null-terminated ASCII strings)

Bone record (56 bytes):

    offset  size  field
    0       4     name offset (byte offset into the file for this bone's name)
    4       4     parent index (i32; -1 = root, else 0-based index into bone array)
    8       48    3x4 row-major transform [rotation_3x3 | translation_3x1]

Each transform row is [rot_x, rot_y, rot_z, translation], giving a 3x3 bind-pose
rotation matrix plus a 3D position per bone. The translation values match
PesSkeletonData._positions exactly; the parent indices match
PesSkeletonData.bones[*].sklParent.
"""

import struct
from dataclasses import dataclass
from typing import Optional


@dataclass
class Bone:
    name: str
    parent_index: int          # -1 for root
    rotation: tuple            # 3x3, row-major (9 floats)
    translation: tuple         # 3 floats

    @property
    def parent(self) -> Optional[int]:
        return self.parent_index if self.parent_index >= 0 else None


class SklFile:
    MAGIC = 12
    RECORD_SIZE = 56

    def __init__(self, bones: list[Bone]):
        self.bones = bones

    @property
    def bone_count(self) -> int:
        return len(self.bones)

    def by_name(self, name: str) -> Optional[Bone]:
        for bone in self.bones:
            if bone.name == name:
                return bone
        return None

    @classmethod
    def read(cls, data: bytes) -> "SklFile":
        magic, bone_count, record_size = struct.unpack_from("<III", data, 0)
        if magic != cls.MAGIC:
            raise ValueError(f"bad SKL magic: {magic} (expected {cls.MAGIC})")
        if record_size != cls.RECORD_SIZE:
            raise ValueError(f"bad SKL record size: {record_size} (expected {cls.RECORD_SIZE})")

        bones = []
        for i in range(bone_count):
            off = 12 + i * record_size
            name_offset, parent_index = struct.unpack_from("<Ii", data, off)
            floats = struct.unpack_from("<12f", data, off + 8)

            # 3x4 row-major: row = [rot_x, rot_y, rot_z, translation]
            rotation = (floats[0], floats[1], floats[2],
                        floats[4], floats[5], floats[6],
                        floats[8], floats[9], floats[10])
            translation = (floats[3], floats[7], floats[11])

            # Resolve name from the name table at name_offset
            end = data.index(b"\x00", name_offset)
            name = data[name_offset:end].decode("ascii")

            bones.append(Bone(name=name, parent_index=parent_index,
                              rotation=rotation, translation=translation))

        return cls(bones)

    @classmethod
    def read_file(cls, path: str) -> "SklFile":
        with open(path, "rb") as f:
            return cls.read(f.read())

    def write(self) -> bytes:
        # Build name table and record each bone's name offset
        name_table = bytearray()
        name_offsets = []
        for bone in self.bones:
            name_offsets.append(len(name_table) + 12 + self.bone_count * self.RECORD_SIZE)
            name_table.extend(bone.name.encode("ascii"))
            name_table.append(0)

        out = bytearray()
        out += struct.pack("<III", self.MAGIC, self.bone_count, self.RECORD_SIZE)

        for i, bone in enumerate(self.bones):
            r = bone.rotation
            t = bone.translation
            # 3x4 row-major: [rot_x, rot_y, rot_z, tx, rot_x, rot_y, rot_z, ty, ...]
            floats = (r[0], r[1], r[2], t[0],
                      r[3], r[4], r[5], t[1],
                      r[6], r[7], r[8], t[2])
            out += struct.pack("<Ii12f", name_offsets[i], bone.parent_index, *floats)

        out += name_table

        # Pad to 4-byte alignment (the template body.skl has 2 trailing nulls)
        while len(out) % 4 != 0:
            out.append(0)

        return bytes(out)

    def write_file(self, path: str) -> None:
        with open(path, "wb") as f:
            f.write(self.write())


if __name__ == "__main__":
    import sys

    path = sys.argv[1] if len(sys.argv) > 1 else "boots.skl"
    skl = SklFile.read_file(path)

    print(f"Bone count: {skl.bone_count}")
    print()
    for i, bone in enumerate(skl.bones[:15]):
        parent = skl.bones[bone.parent].name if bone.parent is not None else "(root)"
        t = bone.translation
        print(f"  {i:3d} {bone.name:24s} parent={parent:24s} "
              f"pos=({t[0]:.6f}, {t[1]:.6f}, {t[2]:.6f})")

    # Roundtrip test
    original = open(path, "rb").read()
    rewritten = skl.write()
    if original == rewritten:
        print(f"\nRoundtrip: OK ({len(original)} bytes, lossless)")
    else:
        print(f"\nRoundtrip: MISMATCH (orig={len(original)} bytes, new={len(rewritten)} bytes)")
