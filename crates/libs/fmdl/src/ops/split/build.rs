//! Component construction: equipresent sets, per-bone storable items, the
//! base-bone heuristic, the principal-axis sort, and the fragment builder.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::format::FmdlError;
use crate::model::{Mesh, Model};

use super::{
    BONE_LIMIT_SOFT, BoneMapping, FACE_LIMIT_SOFT, PREFERRED_BASE_BONES, StorableItems,
    VERTEX_LIMIT_SOFT, bone_mapping, empty_like, push_vertex,
};
use crate::ops::vertex_enc::topological_key;

/// Vertex `index`'s split key: the stored bytes of position, weights and
/// bone indices (all vertices of one mesh share the bone group, so group
/// slot numbers compare correctly here).
fn split_key(mesh: &Mesh, index: usize) -> Vec<u8> {
    topological_key(&mesh.vertices, index)
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

/// A bone under which storable items remain: preferred names first, then
/// every bone; a bone with nothing left under it hands over to its parent.
/// `None` is the whole-mesh root.
fn select_base_bone(
    model: &Model,
    parents: &[Option<usize>],
    items_per_bone: &HashMap<Option<usize>, StorableItems>,
) -> Option<usize> {
    let mut queue: VecDeque<usize> = VecDeque::new();
    for name in PREFERRED_BASE_BONES {
        if let Some(index) = model.bones.iter().position(|bone| bone.name == name) {
            queue.push_back(index);
        }
    }
    queue.extend(0..model.bones.len());

    let mut tried = HashSet::new();
    while let Some(bone) = queue.pop_front() {
        if !tried.insert(bone) {
            continue;
        }
        let empty = items_per_bone
            .get(&Some(bone))
            .is_none_or(|items| items.faces.is_empty() && items.loose.is_empty());
        if empty {
            if let Some(parent) = parents[bone] {
                queue.push_back(parent);
            }
            continue;
        }
        return Some(bone);
    }
    None
}

/// The first principal axis of the vertex cloud in `items`, oriented from
/// `bone_position` toward the cloud's centre. Power iteration on the 3x3
/// covariance matrix, eight iterations.
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
        for a in 0..3 {
            for b in 0..3 {
                covariance[a][b] += (point[a] - mean[a]) * (point[b] - mean[b]);
            }
        }
    }
    // Start from the axis of largest variance so a degenerate cloud still
    // gives a deterministic vector.
    let diagonal = [covariance[0][0], covariance[1][1], covariance[2][2]];
    let mut vector = [0.0f32; 3];
    vector[if diagonal[1] > diagonal[0] {
        if diagonal[2] > diagonal[1] { 2 } else { 1 }
    } else if diagonal[2] > diagonal[0] {
        2
    } else {
        0
    }] = 1.0;
    for _ in 0..8 {
        let mut next = [0.0f32; 3];
        for a in 0..3 {
            for b in 0..3 {
                next[a] += covariance[a][b] * vector[b];
            }
        }
        let norm = (next[0] * next[0] + next[1] * next[1] + next[2] * next[2]).sqrt();
        if norm > 0.0 {
            vector = [next[0] / norm, next[1] / norm, next[2] / norm];
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
        select_base_bone(model, parents, items_per_bone)
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
        let score = |vertex: usize| -> f32 {
            let position = mesh.vertices.positions[vertex];
            position[0] * axis[0] + position[1] * axis[1] + position[2] * axis[2]
        };

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
    let mut vertices = empty_like(&mesh.vertices, vertex_capacity(&sorted_sets, sets));
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let bone_group: Vec<usize> = mesh
        .bone_group
        .iter()
        .copied()
        .filter(|bone| selected_bones.contains(bone))
        .collect();
    let index_of: HashMap<usize, u8> = bone_group
        .iter()
        .enumerate()
        .map(|(slot, &bone)| (bone, slot as u8))
        .collect();
    for &set in &sorted_sets {
        for &vertex in &sets[set] {
            remap.insert(vertex, vertices.positions.len());
            push_vertex(&mut vertices, mesh, vertex, &index_of)?;
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

fn vertex_capacity(sets: &[usize], all_sets: &[Vec<usize>]) -> usize {
    sets.iter().map(|&set| all_sets[set].len()).sum()
}

/// Splits `mesh` into component meshes each under the soft limits.
pub(super) fn split_mesh(
    model: &Model,
    mesh: &Mesh,
    parents: &[Option<usize>],
) -> Result<Vec<Mesh>, FmdlError> {
    let count = mesh.vertices.positions.len();
    for face in &mesh.faces {
        for &index in face {
            let vertex = usize::from(index);
            if vertex >= count {
                return Err(FmdlError::BadReference {
                    what: "face vertex",
                    index: vertex,
                });
            }
        }
    }
    let skinned = mesh.vertices.bone_indices.is_some();
    let mappings: Vec<BoneMapping> = (0..count)
        .map(|index| bone_mapping(mesh, index))
        .collect::<Result<_, _>>()?;

    // Equipresent vertex sets: vertices sharing a split key travel together.
    let mut set_of = vec![0usize; count];
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut key_to_set: HashMap<Vec<u8>, usize> = HashMap::new();
    for (index, slot) in set_of.iter_mut().enumerate() {
        let key = split_key(mesh, index);
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
