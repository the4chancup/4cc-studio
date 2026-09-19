"""Generates `hand_split_wrist.json`: the reference output of Blender's select -> Select More
(once) -> Separate by selection on a synthetic connected-wrist mesh, the semantics the plan's
hand auto-split reproduces in Rust (model_conversion/hand_split.md "Hand auto-split").

Run headless:  blender -b --python hand_split_wrist.py -- hand_split_wrist.json

The mesh (also built by the Rust test, `ops/hand_split.rs`): a 5x3 grid of quads, x = 0..4,
y = 0..2, each quad split along its (x, y)-(x+1, y+1) diagonal, plus one fan vertex V at
(1.5, 3, 0) with two triangles: ((1,2), (2,2), V) and ((2,2), (3,2), V). Weights by column:
x <= 1 -> sk_forearm_l 1.0; x == 2 -> sk_hand_l 1.0; x == 3 -> sk_hand_l 0.5 + skh_index_l 0.5;
x == 4 -> skh_index_l 1.0; V -> sk_hand_l 1.0. So the seeds are columns 3 and 4; Select More
brings in column 2 and V (V shares an edge with (3,2)); the glove gets every face with all
three vertices in {columns 2..4, V}, the body keeps the rest, and column 2 plus V are duplicated.

Faces are written as their three vertex positions in winding order, rotated so the
lexicographically smallest position comes first, then sorted; vertex order is Blender's own and
carries no meaning.
"""
import json
import sys

import bpy

out_path = sys.argv[sys.argv.index("--") + 1]

bpy.ops.wm.read_factory_settings(use_empty=True)

# --- the mesh -------------------------------------------------------------------------------
verts = []
index = {}
for x in range(5):
    for y in range(3):
        index[(x, y)] = len(verts)
        verts.append((float(x), float(y), 0.0))
V = len(verts)
verts.append((1.5, 3.0, 0.0))

faces = []
for x in range(4):
    for y in range(2):
        a, b, c, d = index[(x, y)], index[(x + 1, y)], index[(x + 1, y + 1)], index[(x, y + 1)]
        faces.append((a, b, c))
        faces.append((a, c, d))
faces.append((index[(1, 2)], index[(2, 2)], V))
faces.append((index[(2, 2)], index[(3, 2)], V))

mesh = bpy.data.meshes.new("wrist")
mesh.from_pydata(verts, [], faces)
mesh.update()
obj = bpy.data.objects.new("wrist", mesh)
bpy.context.scene.collection.objects.link(obj)

groups = {name: obj.vertex_groups.new(name=name) for name in ("sk_forearm_l", "sk_hand_l", "skh_index_l")}
for (x, y), i in index.items():
    if x <= 1:
        groups["sk_forearm_l"].add([i], 1.0, "REPLACE")
    elif x == 2:
        groups["sk_hand_l"].add([i], 1.0, "REPLACE")
    elif x == 3:
        groups["sk_hand_l"].add([i], 0.5, "REPLACE")
        groups["skh_index_l"].add([i], 0.5, "REPLACE")
    else:
        groups["skh_index_l"].add([i], 1.0, "REPLACE")
groups["sk_hand_l"].add([V], 1.0, "REPLACE")

# --- select (positive skh_*_l weight), grow once, separate ----------------------------------
skh = [g.index for g in obj.vertex_groups if g.name.startswith("skh_") and g.name.endswith("_l")]
for v in mesh.vertices:
    v.select = any(ge.group in skh and ge.weight > 0.0 for ge in v.groups)
for e in mesh.edges:
    e.select = False
for p in mesh.polygons:
    p.select = False

bpy.context.view_layer.objects.active = obj
obj.select_set(True)
bpy.ops.object.mode_set(mode="EDIT")
bpy.ops.mesh.select_mode(type="VERT")
bpy.ops.mesh.select_more()
bpy.ops.mesh.separate(type="SELECTED")
bpy.ops.object.mode_set(mode="OBJECT")

objects = [o for o in bpy.context.scene.objects if o.type == "MESH"]
assert len(objects) == 2, [o.name for o in objects]
body = next(o for o in objects if o.name == "wrist")
glove = next(o for o in objects if o.name != "wrist")


def face_list(o):
    m = o.data
    out = []
    for p in m.polygons:
        pts = [tuple(round(c, 6) for c in m.vertices[i].co) for i in p.vertices]
        assert len(pts) == 3
        start = min(range(3), key=lambda k: pts[k])
        out.append(pts[start:] + pts[:start])
    return sorted(out)


result = {
    "body": {"vertex_count": len(body.data.vertices), "faces": face_list(body)},
    "glove": {"vertex_count": len(glove.data.vertices), "faces": face_list(glove)},
}
with open(out_path, "w", encoding="utf-8", newline="\n") as f:
    json.dump(result, f, indent=1)
    f.write("\n")
print("wrote", out_path, "body faces", len(result["body"]["faces"]), "glove faces", len(result["glove"]["faces"]))
