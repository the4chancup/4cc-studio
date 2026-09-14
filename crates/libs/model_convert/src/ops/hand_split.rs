//! The hand auto-split: separating glove geometry (vertices weighted to `skh_` hand bones)
//! from a body model. The semantics are Blender's select -> Select More -> Separate by
//! selection, done in process — no geometric cut, no reweighting, positions and weights
//! untouched.

use std::collections::HashMap;

use crate::ir::{
    Bone, CanonicalModel, Mesh, MeshGroup, Vertices, bone_is_weighted, rebuild_bone_list,
    remap_bone_group, validate,
};

/// Which hand a bone or a split part belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    /// `_l` suffixed.
    Left,
    /// `_r` suffixed.
    Right,
}

/// A model split at the wrists: the body keeps every face not fully inside a hand
/// selection; a glove is `Some` only when its hand had positively weighted `skh_`
/// vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct HandSplit {
    /// The model with the hand geometry removed.
    pub body: CanonicalModel,
    /// The left-hand part, when any vertex weights an `skh_*_l` bone.
    pub glove_l: Option<CanonicalModel>,
    /// The right-hand part, when any vertex weights an `skh_*_r` bone.
    pub glove_r: Option<CanonicalModel>,
}

/// Whether `bone` is a hand-skeleton-exclusive bone (`skh_` prefix) and for which hand
/// (`_l` / `_r` suffix); `None` for every other name.
pub fn hand_of(bone: &str) -> Option<Hand> {
    if !bone.starts_with("skh_") {
        return None;
    }
    if bone.ends_with("_l") {
        Some(Hand::Left)
    } else if bone.ends_with("_r") {
        Some(Hand::Right)
    } else {
        None
    }
}

/// Whether any vertex carries a positive weight on a hand bone (the plan's detection
/// rule: names in a bone list do not count, weights do).
pub fn has_hand_weights(ir: &CanonicalModel) -> bool {
    ir.meshes.iter().any(|mesh| {
        let (Some(indices), Some(weights)) =
            (&mesh.vertices.bone_indices, &mesh.vertices.bone_weights)
        else {
            return false;
        };
        indices.iter().zip(weights).any(|(row, ws)| {
            row.iter().enumerate().any(|(slot, &entry)| {
                ws[slot] > 0.0
                    && hand_of(&ir.bones[mesh.bone_group[usize::from(entry)]].name).is_some()
            })
        })
    })
}

/// The topological identity key: position bits, the bone index row, the weight bits —
/// the same fields `fmdl::ops::vertex_enc`'s `topological_key` groups on. UVs are
/// deliberately absent: a UV seam is two entries, one topological vertex.
fn class_key(mesh: &Mesh, index: usize) -> Vec<u8> {
    let vertices = &mesh.vertices;
    let mut key = Vec::new();
    for component in vertices.positions[index] {
        key.extend(component.to_le_bytes());
    }
    if let Some(indices) = &vertices.bone_indices {
        key.extend(indices[index]);
    }
    if let Some(weights) = &vertices.bone_weights {
        for component in weights[index] {
            key.extend(component.to_le_bytes());
        }
    }
    key
}

/// The entry → topological-class map for `mesh`: entries sharing position, bone
/// indices and bone weights are one vertex.
fn classes(mesh: &Mesh) -> Vec<usize> {
    let mut ids: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut class = Vec::with_capacity(mesh.vertices.len());
    for index in 0..mesh.vertices.len() {
        let next = ids.len();
        class.push(*ids.entry(class_key(mesh, index)).or_insert(next));
    }
    class
}

/// The per-class selection for `hand` on `mesh`: seeds (a positive weight on a slot
/// whose group bone is `hand`'s) grown once along faces. `class` is the entry →
/// class map from [`classes`]; the returned vector is indexed by class.
fn selected(mesh: &Mesh, class: &[usize], bones: &[Bone], hand: Hand) -> Vec<bool> {
    let classes = class.iter().copied().max().map_or(0, |max| max + 1);
    let mut sel = vec![false; classes];
    if let (Some(indices), Some(weights)) =
        (&mesh.vertices.bone_indices, &mesh.vertices.bone_weights)
    {
        for (vertex, (row, ws)) in indices.iter().zip(weights).enumerate() {
            sel[class[vertex]] |= row.iter().enumerate().any(|(slot, &entry)| {
                ws[slot] > 0.0
                    && hand_of(&bones[mesh.bone_group[usize::from(entry)]].name) == Some(hand)
            });
        }
    }
    // Grow once: one pass over the faces against the seed set (chaining off a
    // just-grown class would be a flood fill, not one Select More).
    let seeds = sel.clone();
    for face in &mesh.faces {
        if face.iter().any(|&i| seeds[class[usize::from(i)]]) {
            for &i in face {
                sel[class[usize::from(i)]] = true;
            }
        }
    }
    sel
}

/// `mesh` restricted to `faces`, vertices re-indexed in first-use order and every present
/// vertex column copied.
fn part_mesh(mesh: &Mesh, faces: &[[u16; 3]]) -> Mesh {
    let mut remap = vec![u16::MAX; mesh.vertices.len()];
    let mut order: Vec<usize> = Vec::new();
    let faces: Vec<[u16; 3]> = faces
        .iter()
        .map(|face| {
            face.map(|i| {
                let i = usize::from(i);
                match remap[i] {
                    u16::MAX => {
                        let new = order.len() as u16;
                        remap[i] = new;
                        order.push(i);
                        new
                    }
                    new => new,
                }
            })
        })
        .collect();
    fn take<T: Clone>(column: &[T], order: &[usize]) -> Vec<T> {
        order.iter().map(|&i| column[i].clone()).collect()
    }
    let v = &mesh.vertices;
    Mesh {
        vertices: Vertices {
            positions: take(&v.positions, &order),
            normals: v.normals.as_ref().map(|c| take(c, &order)),
            tangents: v.tangents.as_ref().map(|c| take(c, &order)),
            bitangents: v.bitangents.as_ref().map(|c| take(c, &order)),
            colors: v.colors.as_ref().map(|c| take(c, &order)),
            uvs: v.uvs.iter().map(|set| take(set, &order)).collect(),
            uv_high_precision: v.uv_high_precision.clone(),
            bone_indices: v.bone_indices.as_ref().map(|c| take(c, &order)),
            bone_weights: v.bone_weights.as_ref().map(|c| take(c, &order)),
            bone_weight_width: v.bone_weight_width,
        },
        faces,
        bone_group: mesh.bone_group.clone(),
        material: mesh.material,
        extension_headers: mesh.extension_headers.clone(),
        custom_bounding_box: mesh.custom_bounding_box,
    }
}

/// The model made of the mesh parts `parts` gives (`None` drops the mesh): mesh indices
/// in groups remap, a group survives when it lists a surviving mesh or is an ancestor of
/// one, and unweighted bones are pruned. Materials, textures, extension headers and the
/// source format copy whole.
fn assemble(source: &CanonicalModel, parts: Vec<Option<Mesh>>) -> CanonicalModel {
    let mut mesh_of = vec![usize::MAX; source.meshes.len()];
    let mut meshes = Vec::new();
    for (old, part) in parts.into_iter().enumerate() {
        if let Some(mesh) = part {
            mesh_of[old] = meshes.len();
            meshes.push(mesh);
        }
    }
    // Parents precede children, so one reverse pass decides a group's survival.
    let mut keep = vec![false; source.mesh_groups.len()];
    for g in (0..source.mesh_groups.len()).rev() {
        keep[g] = source.mesh_groups[g]
            .meshes
            .iter()
            .any(|&m| mesh_of[m] != usize::MAX)
            || source
                .mesh_groups
                .iter()
                .enumerate()
                .skip(g + 1)
                .any(|(h, child)| keep[h] && child.parent == Some(g));
    }
    let mut group_of = vec![usize::MAX; source.mesh_groups.len()];
    let mut groups = Vec::new();
    for (g, group) in source.mesh_groups.iter().enumerate() {
        if !keep[g] {
            continue;
        }
        group_of[g] = groups.len();
        groups.push(MeshGroup {
            name: group.name.clone(),
            parent: group.parent.map(|p| group_of[p]),
            meshes: group
                .meshes
                .iter()
                .filter_map(|&m| (mesh_of[m] != usize::MAX).then_some(mesh_of[m]))
                .collect(),
            visible: group.visible,
        });
    }
    let mut model = CanonicalModel {
        bones: source.bones.clone(),
        meshes,
        mesh_groups: groups,
        materials: source.materials.clone(),
        textures: source.textures.clone(),
        extension_headers: source.extension_headers.clone(),
        source_format: source.source_format,
    };
    // Prune unused bones; slots pointing at a dropped bone are unweighted by
    // construction and become index 0, weight 0.
    let keep_bone: Vec<bool> = (0..model.bones.len())
        .map(|bone| bone_is_weighted(&model, bone))
        .collect();
    let old_to_new = rebuild_bone_list(&mut model.bones, &keep_bone);
    let no_redirect = vec![None; old_to_new.len()];
    for mesh in &mut model.meshes {
        remap_bone_group(mesh, &old_to_new, &no_redirect);
    }
    validate(&model).expect("hand split keeps a consistent IR");
    model
}

/// The plan's select -> grow once -> separate, per hand, in Rust.
pub fn split_by_skeleton_group(ir: &CanonicalModel) -> HandSplit {
    if !has_hand_weights(ir) {
        return HandSplit {
            body: ir.clone(),
            glove_l: None,
            glove_r: None,
        };
    }
    // Selections run over topological classes: two entries at a UV seam are one vertex,
    // so a class is selected or not, never half.
    let class: Vec<Vec<usize>> = ir.meshes.iter().map(classes).collect();
    // The two hands are independent selections of the input.
    let sel_l: Vec<Vec<bool>> = ir
        .meshes
        .iter()
        .enumerate()
        .map(|(m, mesh)| selected(mesh, &class[m], &ir.bones, Hand::Left))
        .collect();
    let sel_r: Vec<Vec<bool>> = ir
        .meshes
        .iter()
        .enumerate()
        .map(|(m, mesh)| selected(mesh, &class[m], &ir.bones, Hand::Right))
        .collect();
    // A face goes to a glove when all three corners' classes are in its selection;
    // every other face stays with the body.
    let glove_part = |sel: &[Vec<bool>]| -> Vec<Option<Mesh>> {
        ir.meshes
            .iter()
            .enumerate()
            .map(|(m, mesh)| {
                let faces: Vec<[u16; 3]> = mesh
                    .faces
                    .iter()
                    .copied()
                    .filter(|f| f.iter().all(|&i| sel[m][class[m][usize::from(i)]]))
                    .collect();
                (!faces.is_empty()).then(|| part_mesh(mesh, &faces))
            })
            .collect()
    };
    let glove_l = glove_part(&sel_l);
    let glove_r = glove_part(&sel_r);
    let body_parts: Vec<Option<Mesh>> = ir
        .meshes
        .iter()
        .enumerate()
        .map(|(m, mesh)| {
            let faces: Vec<[u16; 3]> = mesh
                .faces
                .iter()
                .copied()
                .filter(|f| {
                    !(f.iter().all(|&i| sel_l[m][class[m][usize::from(i)]])
                        || f.iter().all(|&i| sel_r[m][class[m][usize::from(i)]]))
                })
                .collect();
            (!faces.is_empty()).then(|| part_mesh(mesh, &faces))
        })
        .collect();
    HandSplit {
        body: assemble(ir, body_parts),
        glove_l: glove_l
            .iter()
            .any(Option::is_some)
            .then(|| assemble(ir, glove_l)),
        glove_r: glove_r
            .iter()
            .any(Option::is_some)
            .then(|| assemble(ir, glove_r)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::affine::Affine;
    use crate::ir::{Material, SourceFormat};
    use crate::materials::MaterialFamily;

    /// A bone with no parent and the identity bind.
    fn bone(name: &str) -> Bone {
        Bone {
            name: name.to_string(),
            parent: None,
            matrix: Affine::IDENTITY,
            global_position: None,
            local_position: None,
            bounding_box: None,
        }
    }

    /// A model around `mesh` with one material and one group per mesh.
    fn model(bones: Vec<Bone>, mesh: Mesh) -> CanonicalModel {
        CanonicalModel {
            bones,
            meshes: vec![mesh],
            mesh_groups: vec![MeshGroup {
                name: "group".to_string(),
                parent: None,
                meshes: vec![0],
                visible: true,
            }],
            materials: vec![Material {
                name: "mat".to_string(),
                family: MaterialFamily::Shaded,
                two_sided: None,
                transparent: None,
                antiblur: None,
                textures: vec![],
                parameters: vec![],
                fox: None,
                prefox: None,
            }],
            textures: vec![],
            extension_headers: Default::default(),
            source_format: SourceFormat::Fox,
        }
    }

    /// The mesh of `hand_split_wrist.py`: a 5x3 grid (x 0..4, y 0..2) split along the
    /// (x,y)-(x+1,y+1) diagonals, plus the fan vertex V = (1.5, 3, 0). Weights by column
    /// over the bone group `[sk_forearm_l, sk_hand_l, skh_index_l]`.
    fn wrist_mesh() -> Mesh {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut weights = Vec::new();
        let at = |x: usize, y: usize| (x * 3 + y) as u16;
        for x in 0..5usize {
            for y in 0..3usize {
                positions.push([x as f32, y as f32, 0.0]);
                let (row, ws) = match x {
                    0 | 1 => ([0, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
                    2 => ([1, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
                    3 => ([1, 2, 0, 0], [0.5, 0.5, 0.0, 0.0]),
                    _ => ([2, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
                };
                indices.push(row);
                weights.push(ws);
            }
        }
        let v = positions.len() as u16;
        positions.push([1.5, 3.0, 0.0]);
        indices.push([1, 0, 0, 0]);
        weights.push([1.0, 0.0, 0.0, 0.0]);
        let mut faces = Vec::new();
        for x in 0..4usize {
            for y in 0..2usize {
                let (a, b, c, d) = (at(x, y), at(x + 1, y), at(x + 1, y + 1), at(x, y + 1));
                faces.push([a, b, c]);
                faces.push([a, c, d]);
            }
        }
        faces.push([at(1, 2), at(2, 2), v]);
        faces.push([at(2, 2), at(3, 2), v]);
        Mesh {
            vertices: Vertices {
                positions,
                bone_indices: Some(indices),
                bone_weights: Some(weights),
                bone_weight_width: Some(4),
                ..Vertices::default()
            },
            faces,
            bone_group: vec![0, 1, 2],
            material: 0,
            extension_headers: Default::default(),
            custom_bounding_box: None,
        }
    }

    /// Every face as its three positions rotated smallest-first, the whole list sorted —
    /// the normalization `hand_split_wrist.py` uses.
    fn face_set(part: &CanonicalModel) -> Vec<[[f32; 3]; 3]> {
        let mut out: Vec<[[f32; 3]; 3]> = part
            .meshes
            .iter()
            .flat_map(|mesh| {
                mesh.faces.iter().map(|face| {
                    let pts = face.map(|i| mesh.vertices.positions[usize::from(i)]);
                    let start = (0..3)
                        .min_by(|&a, &b| pts[a].partial_cmp(&pts[b]).expect("no NaN"))
                        .expect("three corners");
                    [pts[start], pts[(start + 1) % 3], pts[(start + 2) % 3]]
                })
            })
            .collect();
        out.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        out
    }

    /// The (index, weight) row of the vertex at `position`.
    fn skinning(part: &CanonicalModel, position: [f32; 3]) -> ([u8; 4], [f32; 4]) {
        for mesh in &part.meshes {
            if let Some(v) = mesh.vertices.positions.iter().position(|p| *p == position) {
                return (
                    mesh.vertices.bone_indices.as_ref().expect("indices")[v],
                    mesh.vertices.bone_weights.as_ref().expect("weights")[v],
                );
            }
        }
        panic!("no vertex at {position:?}");
    }

    #[test]
    fn blender_reference_wrist() {
        let ir = model(
            vec![bone("sk_forearm_l"), bone("sk_hand_l"), bone("skh_index_l")],
            wrist_mesh(),
        );
        let split = split_by_skeleton_group(&ir);
        assert_eq!(split.glove_r, None);
        let glove = split.glove_l.expect("left glove");
        assert_eq!(split.body.meshes[0].vertices.len(), 10);
        assert_eq!(split.body.meshes[0].faces.len(), 9);
        assert_eq!(glove.meshes[0].vertices.len(), 10);
        assert_eq!(glove.meshes[0].faces.len(), 9);
        // The literal face sets of hand_split_wrist.json (the file is the source).
        assert_eq!(
            face_set(&split.body),
            vec![
                [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
                [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 2.0, 0.0]],
                [[0.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 2.0, 0.0]],
                [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
                [[1.0, 0.0, 0.0], [2.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
                [[1.0, 1.0, 0.0], [2.0, 1.0, 0.0], [2.0, 2.0, 0.0]],
                [[1.0, 1.0, 0.0], [2.0, 2.0, 0.0], [1.0, 2.0, 0.0]],
                [[1.0, 2.0, 0.0], [2.0, 2.0, 0.0], [1.5, 3.0, 0.0]],
            ]
        );
        assert_eq!(
            face_set(&glove),
            vec![
                [[1.5, 3.0, 0.0], [2.0, 2.0, 0.0], [3.0, 2.0, 0.0]],
                [[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 1.0, 0.0]],
                [[2.0, 0.0, 0.0], [3.0, 1.0, 0.0], [2.0, 1.0, 0.0]],
                [[2.0, 1.0, 0.0], [3.0, 1.0, 0.0], [3.0, 2.0, 0.0]],
                [[2.0, 1.0, 0.0], [3.0, 2.0, 0.0], [2.0, 2.0, 0.0]],
                [[3.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 1.0, 0.0]],
                [[3.0, 0.0, 0.0], [4.0, 1.0, 0.0], [3.0, 1.0, 0.0]],
                [[3.0, 1.0, 0.0], [4.0, 1.0, 0.0], [4.0, 2.0, 0.0]],
                [[3.0, 1.0, 0.0], [4.0, 2.0, 0.0], [3.0, 2.0, 0.0]],
            ]
        );
        // The duplicated wrist column keeps its full sk_hand_l weight in both parts.
        for (part, want) in [(&split.body, 1u8), (&glove, 0u8)] {
            let (indices, ws) = skinning(part, [2.0, 0.0, 0.0]);
            assert_eq!(ws, [1.0, 0.0, 0.0, 0.0]);
            let entry = mesh_group_bone(part, indices[0]);
            assert_eq!(want, indices[0]);
            assert_eq!(entry, "sk_hand_l");
        }
        assert_eq!(
            split
                .body
                .bones
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>(),
            ["sk_forearm_l", "sk_hand_l"]
        );
        assert_eq!(
            glove
                .bones
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>(),
            ["sk_hand_l", "skh_index_l"]
        );
    }

    /// The bone name a mesh's group entry `slot` indexes.
    fn mesh_group_bone(part: &CanonicalModel, slot: u8) -> &str {
        part.bones[part.meshes[0].bone_group[usize::from(slot)]]
            .name
            .as_str()
    }

    #[test]
    fn detection_follows_weights_not_names() {
        let mut mesh = wrist_mesh();
        // Every vertex weighted to sk_hand_l (group entry 1); the group still names
        // skh_index_l, which must not trigger a split.
        mesh.vertices.bone_indices = Some(vec![[1, 0, 0, 0]; mesh.vertices.len()]);
        mesh.vertices.bone_weights = Some(vec![[1.0, 0.0, 0.0, 0.0]; mesh.vertices.len()]);
        let ir = model(
            vec![bone("sk_forearm_l"), bone("sk_hand_l"), bone("skh_index_l")],
            mesh,
        );
        assert!(!has_hand_weights(&ir));
        let split = split_by_skeleton_group(&ir);
        assert_eq!(split.body, ir);
        assert_eq!(split.glove_l, None);
        assert_eq!(split.glove_r, None);
    }

    #[test]
    fn two_hands_split_independently() {
        // Three disjoint triangles: skh_index_l, skh_index_r, sk_forearm_l.
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut weights = Vec::new();
        let mut faces = Vec::new();
        for (tri, entry) in [1u8, 2, 0].iter().enumerate() {
            let base = positions.len() as u16;
            for corner in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]] {
                positions.push([tri as f32 * 3.0 + corner[0], corner[1], 0.0]);
                indices.push([*entry, 0, 0, 0]);
                weights.push([1.0, 0.0, 0.0, 0.0]);
            }
            faces.push([base, base + 1, base + 2]);
        }
        let mesh = Mesh {
            vertices: Vertices {
                positions,
                bone_indices: Some(indices),
                bone_weights: Some(weights),
                bone_weight_width: Some(4),
                ..Vertices::default()
            },
            faces,
            bone_group: vec![0, 1, 2],
            material: 0,
            extension_headers: Default::default(),
            custom_bounding_box: None,
        };
        let ir = model(
            vec![
                bone("sk_forearm_l"),
                bone("skh_index_l"),
                bone("skh_index_r"),
            ],
            mesh,
        );
        let split = split_by_skeleton_group(&ir);
        for (part, bones) in [
            (&split.body, vec!["sk_forearm_l"]),
            (&split.glove_l.clone().expect("l"), vec!["skh_index_l"]),
            (&split.glove_r.clone().expect("r"), vec!["skh_index_r"]),
        ] {
            assert_eq!(part.meshes.len(), 1);
            assert_eq!(part.meshes[0].faces.len(), 1);
            assert_eq!(part.meshes[0].vertices.len(), 3);
            assert_eq!(
                part.bones
                    .iter()
                    .map(|b| b.name.as_str())
                    .collect::<Vec<_>>(),
                bones
            );
        }
    }

    #[test]
    fn groups_remap_and_empty_ones_drop() {
        let mut ir = model(
            vec![bone("sk_forearm_l"), bone("sk_hand_l"), bone("skh_index_l")],
            wrist_mesh(),
        );
        // A second, body-only mesh so `head` survives on the body side.
        let mut head = wrist_mesh();
        head.vertices.bone_indices = Some(vec![[0, 0, 0, 0]; head.vertices.len()]);
        head.vertices.bone_weights = Some(vec![[1.0, 0.0, 0.0, 0.0]; head.vertices.len()]);
        head.bone_group = vec![0];
        ir.meshes.push(head);
        ir.mesh_groups = vec![
            MeshGroup {
                name: "root".to_string(),
                parent: None,
                meshes: vec![],
                visible: true,
            },
            MeshGroup {
                name: "arm".to_string(),
                parent: Some(0),
                meshes: vec![0],
                visible: true,
            },
            MeshGroup {
                name: "head".to_string(),
                parent: Some(0),
                meshes: vec![1],
                visible: true,
            },
        ];
        let split = split_by_skeleton_group(&ir);
        let glove = split.glove_l.expect("left glove");
        assert_eq!(
            glove
                .mesh_groups
                .iter()
                .map(|g| (g.name.as_str(), g.parent, g.meshes.clone()))
                .collect::<Vec<_>>(),
            [("root", None, vec![]), ("arm", Some(0), vec![0])]
        );
        assert_eq!(
            split
                .body
                .mesh_groups
                .iter()
                .map(|g| (g.name.as_str(), g.parent, g.meshes.clone()))
                .collect::<Vec<_>>(),
            [
                ("root", None, vec![]),
                ("arm", Some(0), vec![0]),
                ("head", Some(0), vec![1]),
            ]
        );
    }

    #[test]
    fn pruned_bones_reparent_to_the_nearest_survivor() {
        let mut bones = vec![
            bone("sk_upperarm_l"),
            bone("sk_forearm_l"),
            bone("sk_hand_l"),
            bone("skh_index_l"),
        ];
        bones[1].parent = Some(0);
        bones[2].parent = Some(1);
        bones[3].parent = Some(2);
        // One glove triangle weighted sk_hand_l + skh_index_l, one disjoint body
        // triangle weighted sk_forearm_l.
        let mesh = Mesh {
            vertices: Vertices {
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [5.0, 0.0, 0.0],
                    [6.0, 0.0, 0.0],
                    [5.0, 1.0, 0.0],
                ],
                bone_indices: Some(vec![
                    [2, 3, 0, 0],
                    [2, 3, 0, 0],
                    [2, 3, 0, 0],
                    [1, 0, 0, 0],
                    [1, 0, 0, 0],
                    [1, 0, 0, 0],
                ]),
                bone_weights: Some(vec![
                    [0.5, 0.5, 0.0, 0.0],
                    [0.5, 0.5, 0.0, 0.0],
                    [0.5, 0.5, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                ]),
                bone_weight_width: Some(4),
                ..Vertices::default()
            },
            faces: vec![[0, 1, 2], [3, 4, 5]],
            bone_group: vec![0, 1, 2, 3],
            material: 0,
            extension_headers: Default::default(),
            custom_bounding_box: None,
        };
        let split = split_by_skeleton_group(&model(bones, mesh));
        let glove = split.glove_l.expect("left glove");
        assert_eq!(
            glove
                .bones
                .iter()
                .map(|b| (b.name.as_str(), b.parent))
                .collect::<Vec<_>>(),
            [("sk_hand_l", None), ("skh_index_l", Some(0))]
        );
        assert_eq!(
            split
                .body
                .bones
                .iter()
                .map(|b| (b.name.as_str(), b.parent))
                .collect::<Vec<_>>(),
            [("sk_forearm_l", None)]
        );
    }

    /// `wrist_mesh` with column 2 duplicated as a UV seam: the copies (indices 16-18)
    /// carry UV `[1.0, 0.0]`; the x=1..2 quads reference the originals, the x=2..3
    /// quads and the fan reference the copies. The last face joins the *originals*
    /// of (2,0)/(2,1) — which no seed-grown face touches — to the fan vertex: entry
    /// identity would leave it unselected, class identity sends it to the glove.
    fn wrist_mesh_uv_seam() -> Mesh {
        let mut mesh = wrist_mesh();
        let at = |x: usize, y: usize| (x * 3 + y) as u16;
        let v = 15u16;
        for orig in [at(2, 0), at(2, 1), at(2, 2)] {
            let i = usize::from(orig);
            let position = mesh.vertices.positions[i];
            let indices = mesh.vertices.bone_indices.as_ref().expect("indices")[i];
            let weights = mesh.vertices.bone_weights.as_ref().expect("weights")[i];
            mesh.vertices.positions.push(position);
            mesh.vertices
                .bone_indices
                .as_mut()
                .expect("indices")
                .push(indices);
            mesh.vertices
                .bone_weights
                .as_mut()
                .expect("weights")
                .push(weights);
        }
        mesh.vertices.uvs = vec![[vec![[0.0, 0.0]; 16], vec![[1.0, 0.0]; 3]].concat()];
        mesh.vertices.uv_high_precision = vec![false];
        let copy = |y: usize| 16 + y as u16;
        // The x=2..3 quads (faces 8..12) and both fan faces (16, 17) use the copies.
        for y in 0..2usize {
            mesh.faces[8 + y * 2] = [copy(y), at(3, y), at(3, y + 1)];
            mesh.faces[8 + y * 2 + 1] = [copy(y), at(3, y + 1), copy(y + 1)];
        }
        mesh.faces[16] = [at(1, 2), copy(2), v];
        mesh.faces[17] = [copy(2), at(3, 2), v];
        mesh.faces.push([at(2, 0), at(2, 1), v]);
        mesh
    }

    #[test]
    fn a_uv_seam_is_one_topological_vertex() {
        let bones = || vec![bone("sk_forearm_l"), bone("sk_hand_l"), bone("skh_index_l")];
        let seamed = split_by_skeleton_group(&model(bones(), wrist_mesh_uv_seam()));
        let plain = split_by_skeleton_group(&model(bones(), wrist_mesh()));
        // The seam changes no topology: the body keeps the Blender reference's face
        // set (asserted literally in `blender_reference_wrist`), and the glove gets
        // that set plus the discriminating face on the column-2 originals.
        assert_eq!(face_set(&seamed.body), face_set(&plain.body));
        let glove_part = seamed.glove_l.expect("left glove");
        let glove_faces = face_set(&glove_part);
        assert_eq!(glove_faces.len(), 10);
        assert!(
            glove_faces.contains(&[[1.5, 3.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.0]]),
            "the face on the column-2 originals landed in the glove"
        );
        for face in face_set(&plain.glove_l.expect("left glove")) {
            assert!(glove_faces.contains(&face), "reference face {face:?}");
        }
        let body = &seamed.body.meshes[0];
        let glove = &glove_part.meshes[0];
        // Body: columns 0-1 (6), column 2's originals (3), the fan's copy (1) and the
        // fan vertex — 11 vertices, 9 faces. Glove: column 2's copies (3), the
        // originals (2,0)/(2,1) on the extra face (2), columns 3-4 (6), the fan
        // vertex — 12 vertices, 10 faces.
        assert_eq!(body.vertices.len(), 11);
        assert_eq!(body.faces.len(), 9);
        assert_eq!(glove.vertices.len(), 12);
        assert_eq!(glove.faces.len(), 10);
        // Each copy lands in the part its faces use: column 2 is body-side at UV
        // [0, 0] on the originals plus [1, 0] for the fan's copy, and glove-side at
        // [1, 0] on the three copies plus [0, 0] on the originals the extra
        // face references.
        let column2_uv = |mesh: &Mesh| -> Vec<[f32; 2]> {
            mesh.vertices
                .positions
                .iter()
                .zip(&mesh.vertices.uvs[0])
                .filter(|(position, _)| position[0] == 2.0)
                .map(|(_, uv)| *uv)
                .collect()
        };
        assert_eq!(
            column2_uv(body),
            vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [1.0, 0.0]]
        );
        assert_eq!(
            column2_uv(glove),
            vec![[1.0, 0.0], [1.0, 0.0], [1.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
        );
    }

    #[test]
    fn a_stale_index_in_an_unweighted_slot_is_safe() {
        // Slot 1's index 5 is past the two-entry group but unweighted; the split's
        // bone-group remap must not look it up.
        let mesh = Mesh {
            vertices: Vertices {
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [5.0, 0.0, 0.0],
                    [6.0, 0.0, 0.0],
                    [5.0, 1.0, 0.0],
                ],
                bone_indices: Some(vec![
                    [0, 5, 0, 0],
                    [0, 5, 0, 0],
                    [0, 5, 0, 0],
                    [1, 0, 0, 0],
                    [1, 0, 0, 0],
                    [1, 0, 0, 0],
                ]),
                bone_weights: Some(vec![
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                ]),
                bone_weight_width: Some(4),
                ..Vertices::default()
            },
            faces: vec![[0, 1, 2], [3, 4, 5]],
            bone_group: vec![0, 1],
            material: 0,
            extension_headers: Default::default(),
            custom_bounding_box: None,
        };
        let split = split_by_skeleton_group(&model(
            vec![bone("sk_forearm_l"), bone("skh_index_l")],
            mesh,
        ));
        let body = &split.body.meshes[0];
        assert_eq!(body.faces.len(), 1);
        assert_eq!(
            body.vertices.bone_indices.as_ref().expect("indices"),
            &vec![[0, 0, 0, 0]; 3]
        );
        assert!(split.glove_l.is_some());
    }

    #[test]
    fn hand_of_names() {
        assert_eq!(hand_of("skh_thumb_mata_l"), Some(Hand::Left));
        assert_eq!(hand_of("skh_index_dip_r"), Some(Hand::Right));
        assert_eq!(hand_of("skf_jaw"), None);
        assert_eq!(hand_of("sk_hand_l"), None);
        assert_eq!(hand_of("skh_x"), None);
    }
}
