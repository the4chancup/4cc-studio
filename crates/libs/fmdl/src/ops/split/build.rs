//! Component construction: equipresent sets, per-bone storable items, the
//! base-bone heuristic, the principal-axis sort, and the fragment builder.

use std::collections::{HashMap, HashSet};

use crate::format::FmdlError;
use crate::model::{Mesh, Model};

use super::{
    BONE_LIMIT_SOFT, BoneMapping, FACE_LIMIT_SOFT, PREFERRED_BASE_BONES, StorableItems,
    VERTEX_LIMIT_SOFT, bone_mapping, empty_like, push_vertex,
};
/// Vertex `index`'s split key: the stored bytes of its position followed
/// by its `(model bone, weight)` pairs — the identity `combine` matches on,
/// so a zero-weight lane's raw index never splits coincident vertices.
fn split_key(mesh: &Mesh, index: usize, mapping: &BoneMapping) -> Vec<u8> {
    let mut key = Vec::new();
    for component in mesh.vertices.positions[index] {
        key.extend(component.to_le_bytes());
    }
    for &(bone, weight) in mapping {
        key.extend(bone.to_le_bytes());
        key.push(weight);
    }
    key
}

/// The static data of one mesh being split: faces as vertex indices, the
/// equipresent vertex sets, and each vertex's model-level bone mapping.
struct Pieces {
    face_vertices: Vec<[usize; 3]>,
    sets: Vec<Vec<usize>>,
    set_of: Vec<usize>,
    mappings: Vec<BoneMapping>,
    skinned: bool,
}

/// Whether `items` fits under the soft limits.
fn fits_in_submesh(items: &StorableItems, pieces: &Pieces) -> bool {
    if items.faces.len() > FACE_LIMIT_SOFT {
        return false;
    }
    let mut used_sets: HashSet<usize> = items.loose.iter().copied().collect();
    for &face in &items.faces {
        for &vertex in &pieces.face_vertices[face] {
            used_sets.insert(pieces.set_of[vertex]);
        }
    }
    let vertex_count: usize = used_sets.iter().map(|&set| pieces.sets[set].len()).sum();
    if vertex_count > VERTEX_LIMIT_SOFT {
        return false;
    }
    if pieces.skinned {
        let mut bones = HashSet::new();
        for &set in &used_sets {
            for &(bone, _) in &pieces.mappings[pieces.sets[set][0]] {
                bones.insert(bone);
            }
        }
        if bones.len() > BONE_LIMIT_SOFT {
            return false;
        }
    }
    true
}

/// A bone under which storable items remain: the first preferred name
/// with items left, else the lowest-index bone with items left, else
/// `None` (the whole-mesh root takes the rest).
fn select_base_bone(
    model: &Model,
    items_per_bone: &HashMap<Option<usize>, StorableItems>,
) -> Option<usize> {
    let has_items = |bone: usize| {
        items_per_bone
            .get(&Some(bone))
            .is_some_and(|items| !items.faces.is_empty() || !items.loose.is_empty())
    };
    for name in PREFERRED_BASE_BONES {
        if let Some(index) = model.bones.iter().position(|bone| bone.name == name)
            && has_items(index)
        {
            return Some(index);
        }
    }
    (0..model.bones.len()).find(|&bone| has_items(bone))
}

/// The first principal axis of the vertex cloud in `items`, oriented from
/// `bone_position` toward the cloud's centre. Power iteration on the 3x3
/// covariance matrix, eight iterations from each basis vector, keeping the
/// result with the largest Rayleigh quotient.
fn sort_vector(points: &[[f32; 3]], bone_position: [f32; 3]) -> [f32; 3] {
    let count = points.len() as f32;
    let mut mean = [0.0f32; 3];
    for point in points {
        for axis in 0..3 {
            mean[axis] += point[axis] / count;
        }
    }
    let mut covariance = [[0.0f32; 3]; 3];
    for point in points {
        let d = [point[0] - mean[0], point[1] - mean[1], point[2] - mean[2]];
        for a in 0..3 {
            for b in 0..3 {
                covariance[a][b] += d[a] * d[b];
            }
        }
    }
    // Power-iterate from each basis vector in turn — a single seed can sit
    // on a non-principal eigenvector — and keep the result with the
    // largest Rayleigh quotient v . (C v) (strictly larger replaces, so a
    // tie keeps the earlier start; a degenerate cloud keeps x).
    let mut vector = [1.0f32, 0.0, 0.0];
    let mut best = f32::NEG_INFINITY;
    for start in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let mut candidate = start;
        for _ in 0..8 {
            let next = multiply(&covariance, candidate);
            let norm = (next[0] * next[0] + next[1] * next[1] + next[2] * next[2]).sqrt();
            if norm > 0.0 {
                candidate = [next[0] / norm, next[1] / norm, next[2] / norm];
            }
        }
        let product = multiply(&covariance, candidate);
        let quotient =
            candidate[0] * product[0] + candidate[1] * product[1] + candidate[2] * product[2];
        if quotient > best {
            best = quotient;
            vector = candidate;
        }
    }
    let mut dot = 0.0f32;
    for axis in 0..3 {
        dot += (mean[axis] - bone_position[axis]) * vector[axis];
    }
    if dot < 0.0 {
        [-vector[0], -vector[1], -vector[2]]
    } else {
        vector
    }
}

/// `matrix` times `vector`: one power-iteration step of `sort_vector`.
fn multiply(matrix: &[[f32; 3]; 3], vector: [f32; 3]) -> [f32; 3] {
    let mut next = [0.0f32; 3];
    for a in 0..3 {
        for b in 0..3 {
            next[a] += matrix[a][b] * vector[b];
        }
    }
    next
}

/// A vertex's projection on `axis`: the score the fragment builder sorts
/// faces and loose sets by.
fn projection(position: [f32; 3], axis: [f32; 3]) -> f32 {
    position[0] * axis[0] + position[1] * axis[1] + position[2] * axis[2]
}

/// Splits off one component from `items_per_bone`; returns it together with
/// the face and loose-set indices it consumed.
fn build_component(
    model: &Model,
    mesh: &Mesh,
    parents: &[Option<usize>],
    items_per_bone: &HashMap<Option<usize>, StorableItems>,
    pieces: &Pieces,
) -> Result<(Mesh, StorableItems), FmdlError> {
    let face_vertices = &pieces.face_vertices;
    let sets = &pieces.sets;
    let set_of = &pieces.set_of;
    let mappings = &pieces.mappings;
    let skinned = pieces.skinned;

    let base_bone = if skinned {
        select_base_bone(model, items_per_bone)
    } else {
        None
    };
    let mut taken;
    let mut selected_bones = HashSet::new();
    let mut selected_sets = HashSet::new();

    if fits_in_submesh(&items_per_bone[&base_bone], pieces) {
        // Climb to the highest ancestor that still fits a single component.
        let mut bone = base_bone;
        taken = items_per_bone[&bone].clone();
        while let Some(current) = bone {
            let parent = parents[current];
            let candidate = &items_per_bone[&parent];
            if !fits_in_submesh(candidate, pieces) {
                break;
            }
            bone = parent;
            taken = candidate.clone();
        }
        for &face in &taken.faces {
            for &vertex in &face_vertices[face] {
                selected_sets.insert(set_of[vertex]);
            }
        }
        for &set in &taken.loose {
            selected_sets.insert(set);
        }
        if skinned {
            for &set in &selected_sets {
                for &(bone_index, _) in &mappings[sets[set][0]] {
                    selected_bones.insert(bone_index);
                }
            }
        }
    } else {
        // A fragment: faces in decreasing order of their best projection on
        // the sort vector, added while the soft limits hold.
        let storable = &items_per_bone[&base_bone];
        let mut points = Vec::new();
        for &face in &storable.faces {
            for &vertex in &face_vertices[face] {
                points.push(mesh.vertices.positions[vertex]);
            }
        }
        for &set in &storable.loose {
            points.push(mesh.vertices.positions[sets[set][0]]);
        }
        let bone_position = base_bone.map_or([0.0; 3], |bone| {
            let position = model.bones[bone].world_position;
            [position[0], position[1], position[2]]
        });
        let axis = sort_vector(&points, bone_position);
        let score = |vertex: usize| -> f32 { projection(mesh.vertices.positions[vertex], axis) };

        let mut faces: Vec<usize> = storable.faces.iter().copied().collect();
        faces.sort_by(|a, b| {
            let score_a = face_vertices[*a]
                .iter()
                .map(|&vertex| score(vertex))
                .fold(f32::MIN, f32::max);
            let score_b = face_vertices[*b]
                .iter()
                .map(|&vertex| score(vertex))
                .fold(f32::MIN, f32::max);
            score_b.total_cmp(&score_a).then(a.cmp(b))
        });
        let mut loose: Vec<usize> = storable.loose.iter().copied().collect();
        loose.sort_by(|a, b| {
            score(sets[*b][0])
                .total_cmp(&score(sets[*a][0]))
                .then(a.cmp(b))
        });

        let mut taken_faces = HashSet::new();
        let mut taken_loose = HashSet::new();
        let mut vertex_count = 0usize;
        for face in faces {
            if taken_faces.len() >= FACE_LIMIT_SOFT {
                break;
            }
            let mut added_bones = HashSet::new();
            let mut added_sets = HashSet::new();
            let mut added_count = 0usize;
            for &vertex in &face_vertices[face] {
                let set = set_of[vertex];
                if !selected_sets.contains(&set) && added_sets.insert(set) {
                    added_count += sets[set].len();
                    for &(bone, _) in &mappings[vertex] {
                        if !selected_bones.contains(&bone) {
                            added_bones.insert(bone);
                        }
                    }
                }
            }
            if selected_bones.len() + added_bones.len() <= BONE_LIMIT_SOFT
                && vertex_count + added_count <= VERTEX_LIMIT_SOFT
            {
                taken_faces.insert(face);
                selected_bones.extend(added_bones);
                selected_sets.extend(added_sets);
                vertex_count += added_count;
            }
        }
        for set in loose {
            if vertex_count >= VERTEX_LIMIT_SOFT {
                break;
            }
            let mut added_bones = HashSet::new();
            for &(bone, _) in &mappings[sets[set][0]] {
                if !selected_bones.contains(&bone) {
                    added_bones.insert(bone);
                }
            }
            if selected_bones.len() + added_bones.len() <= BONE_LIMIT_SOFT
                && vertex_count + sets[set].len() <= VERTEX_LIMIT_SOFT
            {
                taken_loose.insert(set);
                selected_bones.extend(added_bones);
                selected_sets.insert(set);
                vertex_count += sets[set].len();
            }
        }
        // Guarantee progress: a single face bigger than the soft limits
        // still goes out as its own component.
        if taken_faces.is_empty() && taken_loose.is_empty() {
            let storable = &items_per_bone[&base_bone];
            if let Some(&face) = storable.faces.iter().next() {
                taken_faces.insert(face);
                for &vertex in &face_vertices[face] {
                    let set = set_of[vertex];
                    if selected_sets.insert(set) {
                        for &(bone, _) in &mappings[vertex] {
                            selected_bones.insert(bone);
                        }
                    }
                }
            } else if let Some(&set) = storable.loose.iter().next() {
                taken_loose.insert(set);
                selected_sets.insert(set);
                for &(bone, _) in &mappings[sets[set][0]] {
                    selected_bones.insert(bone);
                }
            }
        }
        taken = StorableItems {
            faces: taken_faces,
            loose: taken_loose,
        };
    }

    // Emit the component: each selected equipresent set in the order of its
    // first member, its members in source order (the Nth-occurrence rule
    // needs identical vertices in identical order in every component).
    let mut sorted_sets: Vec<usize> = selected_sets.iter().copied().collect();
    sorted_sets.sort_by_key(|&set| sets[set][0]);
    let mut vertices = empty_like(&mesh.vertices);
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let bone_group: Vec<usize> = mesh
        .bone_group
        .iter()
        .copied()
        .filter(|bone| selected_bones.contains(bone))
        .collect();
    let mut index_of: HashMap<usize, u8> = HashMap::new();
    for (slot, &bone) in bone_group.iter().enumerate() {
        let slot = u8::try_from(slot).map_err(|_| FmdlError::TooManyBones(bone_group.len()))?;
        index_of.insert(bone, slot);
    }
    for &set in &sorted_sets {
        for &vertex in &sets[set] {
            remap.insert(vertex, vertices.positions.len());
            push_vertex(&mut vertices, mesh, vertex, &index_of);
        }
    }
    let mut faces = Vec::with_capacity(taken.faces.len());
    let mut taken_faces: Vec<usize> = taken.faces.iter().copied().collect();
    taken_faces.sort_unstable();
    for face in taken_faces {
        faces.push(face_vertices[face].map(|vertex| remap[&vertex] as u16));
    }

    let component = Mesh {
        vertices,
        faces,
        bone_group,
        material: mesh.material,
        alpha_flags: mesh.alpha_flags,
        shadow_flags: mesh.shadow_flags,
        has_antiblur_meshes: mesh.has_antiblur_meshes,
        is_antiblur_mesh: mesh.is_antiblur_mesh,
        custom_bounding_box: mesh.custom_bounding_box,
    };
    Ok((component, taken))
}

/// Splits `mesh` into component meshes each under the soft limits.
pub(super) fn split_mesh(
    model: &Model,
    mesh: &Mesh,
    parents: &[Option<usize>],
) -> Result<Vec<Mesh>, FmdlError> {
    let count = mesh.vertices.positions.len();
    let skinned = mesh.vertices.bone_indices.is_some();
    let mappings: Vec<BoneMapping> = (0..count)
        .map(|index| bone_mapping(mesh, index))
        .collect::<Result<_, _>>()?;

    // Equipresent vertex sets: vertices sharing a split key travel together.
    let mut set_of = vec![0usize; count];
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut key_to_set: HashMap<Vec<u8>, usize> = HashMap::new();
    for (index, slot) in set_of.iter_mut().enumerate() {
        let key = split_key(mesh, index, &mappings[index]);
        match key_to_set.get(&key) {
            Some(&set) => {
                sets[set].push(index);
                *slot = set;
            }
            None => {
                key_to_set.insert(key, sets.len());
                sets.push(vec![index]);
                *slot = sets.len() - 1;
            }
        }
    }

    // Loose sets: no member referenced by any face.
    let mut used = vec![false; sets.len()];
    for face in &mesh.faces {
        for index in face {
            used[set_of[usize::from(*index)]] = true;
        }
    }
    let loose_sets: HashSet<usize> = (0..sets.len()).filter(|&set| !used[set]).collect();

    // For every bone, the faces and loose sets referencing it or one of its
    // descendants; `None` buckets the whole mesh.
    let mut items_per_bone: HashMap<Option<usize>, StorableItems> = HashMap::new();
    items_per_bone.insert(
        None,
        StorableItems {
            faces: (0..mesh.faces.len()).collect(),
            loose: loose_sets.iter().copied().collect(),
        },
    );
    if skinned {
        for bone in 0..model.bones.len() {
            items_per_bone.insert(Some(bone), StorableItems::default());
        }
        for (face_index, face) in mesh.faces.iter().enumerate() {
            for &vertex_index in face {
                for &(bone, _) in &mappings[usize::from(vertex_index)] {
                    let mut current = Some(bone);
                    while let Some(ancestor) = current {
                        let items = items_per_bone.get_mut(&Some(ancestor)).ok_or(
                            FmdlError::VertexMismatch("split parents index out of range"),
                        )?;
                        if !items.faces.insert(face_index) {
                            break;
                        }
                        current = parents[ancestor];
                    }
                }
            }
        }
        for &set in &loose_sets {
            for &(bone, _) in &mappings[sets[set][0]] {
                let mut current = Some(bone);
                while let Some(ancestor) = current {
                    let items = items_per_bone.get_mut(&Some(ancestor)).ok_or(
                        FmdlError::VertexMismatch("split parents index out of range"),
                    )?;
                    if !items.loose.insert(set) {
                        break;
                    }
                    current = parents[ancestor];
                }
            }
        }
    }

    let pieces = Pieces {
        face_vertices: mesh
            .faces
            .iter()
            .map(|face| face.map(usize::from))
            .collect(),
        sets,
        set_of,
        mappings,
        skinned,
    };
    let mut components = Vec::new();
    loop {
        let root = &items_per_bone[&None];
        if root.faces.is_empty() && root.loose.is_empty() {
            break;
        }
        let (component, taken) = build_component(model, mesh, parents, &items_per_bone, &pieces)?;
        for items in items_per_bone.values_mut() {
            for face in &taken.faces {
                items.faces.remove(face);
            }
            for set in &taken.loose {
                items.loose.remove(set);
            }
        }
        components.push(component);
    }
    Ok(components)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Bone, BoundingBox, Extensions};

    fn bone(name: &str, parent: Option<usize>) -> Bone {
        Bone {
            name: name.to_owned(),
            parent,
            bounding_box: BoundingBox {
                max: [0.0; 4],
                min: [0.0; 4],
            },
            local_position: [0.0; 4],
            world_position: [0.0; 4],
        }
    }

    fn model_with(bones: Vec<Bone>) -> Model {
        Model {
            bones,
            materials: Vec::new(),
            meshes: Vec::new(),
            mesh_groups: Vec::new(),
            extensions: Extensions::default(),
            bone_matrices: None,
        }
    }

    /// `StorableItems` holding `faces` of a mesh's face list.
    fn items_with(faces: impl IntoIterator<Item = usize>) -> StorableItems {
        StorableItems {
            faces: faces.into_iter().collect(),
            loose: HashSet::new(),
        }
    }

    #[test]
    fn projection_is_the_dot_product() {
        // `2 * 4` distinguishes the product from `2 / 4` (the old
        // `[0.5, -1, 2]` axis could not).
        assert_eq!(projection([1.0, 2.0, 3.0], [0.5, 4.0, 2.0]), 14.5);
    }

    #[test]
    fn multiply_is_matrix_times_vector() {
        // Non-symmetric on purpose: a dropped/add-minus term shows.
        let matrix = [[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [5.0, 6.0, 0.0]];
        assert_eq!(multiply(&matrix, [1.0, 0.5, -1.0]), [-1.0, -3.5, 8.0]);
    }

    #[test]
    fn sort_vector_points_along_the_cloud() {
        let o = [5.0, -3.0, 2.0];
        let d = [1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0];
        let points: Vec<[f32; 3]> = (0..10)
            .map(|t| {
                let t = t as f32;
                [o[0] + t * d[0], o[1] + t * d[1], o[2] + t * d[2]]
            })
            .collect();
        // Bone behind the cloud: the axis runs from the bone into it.
        let behind = [o[0] - d[0], o[1] - d[1], o[2] - d[2]];
        let vector = sort_vector(&points, behind);
        let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
        assert!((length - 1.0).abs() < 1e-5);
        for axis in 0..3 {
            assert!((vector[axis] - d[axis]).abs() < 1e-5);
        }
        // Bone past the far end: the axis points back at the cloud.
        let ahead = [o[0] + 20.0 * d[0], o[1] + 20.0 * d[1], o[2] + 20.0 * d[2]];
        let vector = sort_vector(&points, ahead);
        for axis in 0..3 {
            assert!((vector[axis] + d[axis]).abs() < 1e-5);
        }
        // A degenerate cloud picks the first axis deterministically.
        let same = vec![[5.0, -3.0, 2.0]; 5];
        assert_eq!(sort_vector(&same, [0.0, 0.0, 0.0]), [1.0, 0.0, 0.0]);
        // A cloud spread along a single axis picks that axis.
        for axis in 0..3 {
            let points: Vec<[f32; 3]> = (0..10)
                .map(|t| {
                    let mut point = [0.0; 3];
                    point[axis] = t as f32;
                    point
                })
                .collect();
            let vector = sort_vector(&points, [-1.0, -1.0, -1.0]);
            let mut expected = [0.0; 3];
            expected[axis] = 1.0;
            assert_eq!(vector, expected);
        }
    }

    #[test]
    fn sort_vector_finds_the_principal_axis() {
        // Covariance [[18,0,0],[0,10,10],[0,10,10]]: the principal axis is
        // (0, 1, 1)/sqrt(2) (eigenvalue 20), not x (18) — a seed on the
        // largest diagonal alone would sit on x.
        let points = vec![
            [3.0, 0.0, 0.0],
            [-3.0, 0.0, 0.0],
            [0.0, 2.0, 2.0],
            [0.0, -2.0, -2.0],
            [0.0, 1.0, 1.0],
            [0.0, -1.0, -1.0],
        ];
        let vector = sort_vector(&points, [0.0, -1.0, -1.0]);
        let s = 1.0f32 / 2.0f32.sqrt();
        for axis in 0..3 {
            assert!(
                (vector[axis] - [0.0, s, s][axis]).abs() < 1e-5,
                "{vector:?} is not (0, 1, 1)/sqrt(2)"
            );
        }
    }

    #[test]
    fn sort_vector_breaks_degenerate_ties() {
        // Equal y/z variance with no covariance: the deterministic pick
        // is y.
        let mut points = Vec::new();
        for t in -5..=5 {
            points.push([0.0, t as f32, 0.0]);
            points.push([0.0, 0.0, t as f32]);
        }
        assert_eq!(sort_vector(&points, [-1.0, -1.0, -1.0]), [0.0, 1.0, 0.0]);

        // Doubling the z arm makes z the principal axis.
        let mut points = Vec::new();
        for t in -5..=5 {
            points.push([0.0, t as f32, 0.0]);
            points.push([0.0, 0.0, 2.0 * t as f32]);
        }
        assert_eq!(sort_vector(&points, [-1.0, -1.0, -1.0]), [0.0, 0.0, 1.0]);

        // An x-only cloud stays x: a far-off bone only orients the axis
        // (a bad orientation computation flips this), and a bone at the
        // mean — a zero dot — keeps the sign.
        let points: Vec<[f32; 3]> = (-5..=5).map(|t| [t as f32, 0.0, 0.0]).collect();
        assert_eq!(sort_vector(&points, [-1.0, 100.0, 100.0]), [1.0, 0.0, 0.0]);
        assert_eq!(sort_vector(&points, [0.0, 0.0, 0.0]), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn select_base_bone_prefers_named_then_lowest() {
        let items_at = |model: &Model, bones: &[usize]| {
            let mut items: HashMap<Option<usize>, StorableItems> = HashMap::new();
            items.insert(None, StorableItems::default());
            for bone in 0..model.bones.len() {
                items.insert(
                    Some(bone),
                    if bones.contains(&bone) {
                        items_with([0])
                    } else {
                        StorableItems::default()
                    },
                );
            }
            items
        };

        // A preferred bone with items beats a lower-indexed one.
        let model = model_with(vec![
            bone("x0", None),
            bone("sk_foot_l", Some(0)),
            bone("x2", Some(1)),
        ]);
        let items = items_at(&model, &[0, 1]);
        assert_eq!(select_base_bone(&model, &items), Some(1));

        // A preferred bone with no items falls back to the lowest index.
        let items = items_at(&model, &[0]);
        assert_eq!(select_base_bone(&model, &items), Some(0));

        // Nothing anywhere.
        let items = items_at(&model, &[]);
        assert_eq!(select_base_bone(&model, &items), None);

        // `sk_foot_l2` is not a preferred name: bone 0 wins on index.
        let model = model_with(vec![
            bone("x0", None),
            bone("sk_foot_l2", Some(0)),
            bone("x2", Some(1)),
        ]);
        let items = items_at(&model, &[0, 1]);
        assert_eq!(select_base_bone(&model, &items), Some(0));
    }

    #[test]
    fn fits_in_submesh_holds_the_soft_limits() {
        let make_pieces =
            |sets: Vec<Vec<usize>>, mappings: Vec<BoneMapping>, skinned: bool| Pieces {
                face_vertices: Vec::new(),
                set_of: vec![0; 65_535],
                sets,
                mappings,
                skinned,
            };

        // Faces at and one past the soft limit.
        let mut pieces = make_pieces(vec![vec![0]], vec![Vec::new()], false);
        pieces.face_vertices = vec![[0, 0, 0]; FACE_LIMIT_SOFT + 1];
        let items = items_with(0..FACE_LIMIT_SOFT);
        assert!(fits_in_submesh(&items, &pieces));
        let items = items_with(0..FACE_LIMIT_SOFT + 1);
        assert!(!fits_in_submesh(&items, &pieces));

        // Vertices: one loose set holding the whole count.
        let pieces = make_pieces(
            vec![(0..VERTEX_LIMIT_SOFT).collect()],
            vec![Vec::new()],
            false,
        );
        let mut items = StorableItems::default();
        items.loose.insert(0);
        assert!(fits_in_submesh(&items, &pieces));
        let pieces = make_pieces(
            vec![(0..VERTEX_LIMIT_SOFT + 1).collect()],
            vec![Vec::new()],
            false,
        );
        assert!(!fits_in_submesh(&items, &pieces));

        // Bones: one set mapped to 30 vs 31 distinct bones.
        let at = |count: usize| {
            (
                vec![vec![0]],
                vec![(0..count).map(|bone| (bone, 255u8)).collect()],
                true,
            )
        };
        let (sets, mappings, skinned) = at(BONE_LIMIT_SOFT);
        let pieces = make_pieces(sets, mappings, skinned);
        assert!(fits_in_submesh(&items, &pieces));
        let (sets, mappings, skinned) = at(BONE_LIMIT_SOFT + 1);
        let pieces = make_pieces(sets, mappings, skinned);
        assert!(!fits_in_submesh(&items, &pieces));
    }
}
