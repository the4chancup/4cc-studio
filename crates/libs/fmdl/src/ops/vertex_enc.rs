//! Vertex-loop preservation: FMDL stores vertex/face geometry only, so a
//! Blender-style vertex with several loops becomes several FMDL vertices
//! sharing a *topological key* (position, bone weights, bone indices, as
//! the bytes the file stores). The vertex/loop relation is encoded in the
//! vertex order: consecutive vertices with equal topological keys and
//! strictly increasing *nontopological encoding* (normal, color, uv maps in
//! order, tangent — the stored bytes concatenated) are loops of one vertex.
//! The `vertex-loop-preservation` extension flag marks files written under
//! this convention.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::format::f16::f32_to_f16;
use crate::format::{FmdlError, MeshVertices};
use crate::model::{Mesh, Model};

/// For every vertex of `mesh`, the index of the vertex it is a loop of
/// (its own index when it starts a run), read off the ordering convention.
/// The mesh is not changed; meaningful when
/// `Model::extensions.vertex_loop_preservation` is set, otherwise every
/// vertex owns itself except where the convention happens to hold by
/// chance.
pub fn decode(mesh: &Mesh) -> Vec<usize> {
    let vertices = &mesh.vertices;
    let count = vertices.positions.len();
    let topological: Vec<Vec<u8>> = (0..count)
        .map(|index| topological_key(vertices, index))
        .collect();
    let nontopological: Vec<Vec<u8>> = (0..count)
        .map(|index| nontopological_encoding(vertices, index))
        .collect();
    let mut owner = Vec::with_capacity(count);
    for index in 0..count {
        if index > 0
            && topological[index] == topological[index - 1]
            && nontopological[index - 1] < nontopological[index]
        {
            owner.push(owner[index - 1]);
        } else {
            owner.push(index);
        }
    }
    owner
}

/// Reorders `mesh.vertices` and remaps `mesh.faces` so that, per `owner`
/// (same shape as `decode`'s result), the loops of one vertex are
/// contiguous in increasing nontopological order, identical loops collapse
/// to one, and distinct vertices that share a topological key are ordered
/// by decreasing first loop encoding (so they are never read as one
/// vertex). Returns the owner map of the new order. `owner.len() != vertex
/// count` or an owner index out of range -> `FmdlError::VertexMismatch`.
pub fn encode(mesh: &mut Mesh, owner: &[usize]) -> Result<Vec<usize>, FmdlError> {
    let count = mesh.vertices.positions.len();
    if owner.len() != count {
        return Err(FmdlError::VertexMismatch(
            "owner map length does not match vertex count",
        ));
    }
    if owner.iter().any(|index| *index >= count) {
        return Err(FmdlError::VertexMismatch("owner index out of range"));
    }
    let vertices = &mesh.vertices;
    let topological: Vec<Vec<u8>> = (0..count)
        .map(|index| topological_key(vertices, index))
        .collect();
    let nontopological: Vec<Vec<u8>> = (0..count)
        .map(|index| nontopological_encoding(vertices, index))
        .collect();

    // Owner -> its loops, in original order. Identical loops collapse to
    // the first occurrence; survivors sort by increasing encoding.
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, vertex) in owner.iter().enumerate() {
        members.entry(*vertex).or_default().push(index);
    }
    let mut collapsed_to: Vec<usize> = (0..count).collect();
    for loops in members.values_mut() {
        let mut seen: HashMap<&[u8], usize> = HashMap::new();
        let mut kept = Vec::with_capacity(loops.len());
        for index in loops.iter() {
            match seen.get(nontopological[*index].as_slice()) {
                Some(kept_index) => collapsed_to[*index] = *kept_index,
                None => {
                    seen.insert(&nontopological[*index], *index);
                    kept.push(*index);
                }
            }
        }
        kept.sort_by(|a, b| nontopological[*a].cmp(&nontopological[*b]));
        *loops = kept;
    }

    // Topological key -> owners, ordered by decreasing first-loop encoding
    // so that distinct vertices can never read as one vertex's loops.
    let mut key_owners: HashMap<&[u8], Vec<usize>> = HashMap::new();
    for vertex in members.keys() {
        key_owners
            .entry(&topological[*vertex])
            .or_default()
            .push(*vertex);
    }
    for owners in key_owners.values_mut() {
        owners.sort_by(|a, b| nontopological[members[b][0]].cmp(&nontopological[members[a][0]]));
    }

    // Emit in original order: the first time a topological key appears,
    // all its owners' loops come out contiguously.
    let mut order: Vec<usize> = Vec::with_capacity(count);
    let mut new_owner: Vec<usize> = Vec::with_capacity(count);
    let mut seen_keys: HashSet<&[u8]> = HashSet::new();
    for index in 0..count {
        if seen_keys.insert(&topological[index]) {
            for vertex in &key_owners[topological[index].as_slice()] {
                let run_start = order.len();
                for loop_index in &members[vertex] {
                    order.push(*loop_index);
                    new_owner.push(run_start);
                }
            }
        }
    }

    let mut new_index = vec![0usize; count];
    for (index, vertex) in order.iter().enumerate() {
        new_index[*vertex] = index;
    }

    fn permute<T: Clone>(values: &mut Vec<T>, order: &[usize]) {
        *values = order.iter().map(|index| values[*index].clone()).collect();
    }
    let vertices = &mut mesh.vertices;
    permute(&mut vertices.positions, &order);
    if let Some(normals) = &mut vertices.normals {
        permute(normals, &order);
    }
    if let Some(tangents) = &mut vertices.tangents {
        permute(tangents, &order);
    }
    if let Some(colors) = &mut vertices.colors {
        permute(colors, &order);
    }
    for uvs in &mut vertices.uvs {
        permute(uvs, &order);
    }
    if let Some(weights) = &mut vertices.bone_weights {
        permute(weights, &order);
    }
    if let Some(indices) = &mut vertices.bone_indices {
        permute(indices, &order);
    }
    for face in &mut mesh.faces {
        for index in face.iter_mut() {
            *index = new_index[collapsed_to[usize::from(*index)]] as u16;
        }
    }
    Ok(new_owner)
}

/// `encode` on every mesh (owners per mesh) and sets
/// `extensions.vertex_loop_preservation`.
pub fn encode_model(
    model: &mut Model,
    owners: &[Vec<usize>],
) -> Result<Vec<Vec<usize>>, FmdlError> {
    if owners.len() != model.meshes.len() {
        return Err(FmdlError::VertexMismatch(
            "owner maps do not match the mesh count",
        ));
    }
    let mut result = Vec::with_capacity(owners.len());
    for (mesh, owner) in model.meshes.iter_mut().zip(owners) {
        result.push(encode(mesh, owner)?);
    }
    model.extensions.vertex_loop_preservation = true;
    Ok(result)
}

/// `decode` on every mesh when the extension is declared; identity maps
/// otherwise.
pub fn decode_model(model: &Model) -> Vec<Vec<usize>> {
    model
        .meshes
        .iter()
        .map(|mesh| {
            if model.extensions.vertex_loop_preservation {
                decode(mesh)
            } else {
                (0..mesh.vertices.positions.len()).collect()
            }
        })
        .collect()
}

/// Position, bone weights and bone indices as the bytes the file stores.
fn topological_key(vertices: &MeshVertices, index: usize) -> Vec<u8> {
    let mut key = Vec::new();
    for component in vertices.positions[index] {
        key.extend(component.to_le_bytes());
    }
    if let (Some(weights), Some(indices)) = (&vertices.bone_weights, &vertices.bone_indices) {
        key.extend(weights[index]);
        key.extend(indices[index]);
    }
    key
}

/// Normal, color, uv maps in order, tangent — the stored bytes,
/// concatenated.
fn nontopological_encoding(vertices: &MeshVertices, index: usize) -> Vec<u8> {
    let mut encoding = Vec::new();
    if let Some(normals) = &vertices.normals {
        for component in normals[index] {
            encoding.extend(f32_to_f16(component).to_le_bytes());
        }
    }
    if let Some(colors) = &vertices.colors {
        encoding.extend(colors[index]);
    }
    for (map, uvs) in vertices.uvs.iter().enumerate() {
        if vertices
            .uv_high_precision
            .get(map)
            .copied()
            .unwrap_or(false)
        {
            encoding.extend(uvs[index][0].to_le_bytes());
            encoding.extend(uvs[index][1].to_le_bytes());
        } else {
            encoding.extend(f32_to_f16(uvs[index][0]).to_le_bytes());
            encoding.extend(f32_to_f16(uvs[index][1]).to_le_bytes());
        }
    }
    if let Some(tangents) = &vertices.tangents {
        for component in tangents[index] {
            encoding.extend(f32_to_f16(component).to_le_bytes());
        }
    }
    encoding
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FmdlFile;
    use std::collections::BTreeSet;

    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const FIXTURES: &[&[u8]] = &[
        include_bytes!("../../tests/fixtures/konami_highneck.fmdl"),
        include_bytes!("../../tests/fixtures/konami_mouth.fmdl"),
        include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl"),
        ORAL,
        include_bytes!("../../tests/fixtures/addon_placeholder.fmdl"),
    ];

    fn load(bytes: &[u8]) -> Model {
        Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
    }

    /// The partition an owner map describes: owner -> its members.
    fn partition(owner: &[usize]) -> BTreeSet<BTreeSet<usize>> {
        let mut groups: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        for (index, vertex) in owner.iter().enumerate() {
            groups.entry(*vertex).or_default().insert(index);
        }
        groups.values().cloned().collect()
    }

    #[test]
    fn decode_maps_are_well_formed() {
        for bytes in FIXTURES {
            let model = load(bytes);
            for mesh in &model.meshes {
                let owner = decode(mesh);
                for (index, vertex) in owner.iter().enumerate() {
                    assert!(*vertex <= index);
                    assert_eq!(owner[*vertex], *vertex);
                }
                // Each run of equal owners is contiguous.
                for index in 1..owner.len() {
                    if owner[index] == owner[index - 1] {
                        continue;
                    }
                    assert!(!owner[index..].contains(&owner[index - 1]));
                }
            }
        }
    }

    /// A synthetic unskinned mesh: A0, B0, A1, B1 where A's vertices share
    /// a position and differ in uv, B's likewise.
    fn loop_mesh(identical_a: bool) -> Mesh {
        let position_a = [1.0f32, 2.0, 3.0];
        let position_b = [4.0f32, 5.0, 6.0];
        let uv = |u: f32| [u, 0.25f32];
        Mesh {
            vertices: MeshVertices {
                positions: vec![position_a, position_b, position_a, position_b],
                normals: Some(vec![[0.0, 0.0, 1.0, 0.0]; 4]),
                uvs: vec![vec![
                    uv(0.0),
                    uv(0.0),
                    if identical_a { uv(0.0) } else { uv(1.0) },
                    uv(1.0),
                ]],
                uv_high_precision: vec![true],
                ..MeshVertices::default()
            },
            faces: vec![[0, 1, 2], [1, 3, 2]],
            bone_group: Vec::new(),
            material: 0,
            alpha_flags: 0,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        }
    }

    /// The (position, uv) each face corner resolves to, as bit patterns so
    /// the tuples order.
    fn face_tuples(mesh: &Mesh) -> BTreeSet<Vec<([u32; 3], [u32; 2])>> {
        mesh.faces
            .iter()
            .map(|face| {
                face.iter()
                    .map(|index| {
                        (
                            mesh.vertices.positions[usize::from(*index)].map(f32::to_bits),
                            mesh.vertices.uvs[0][usize::from(*index)].map(f32::to_bits),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn encode_groups_loops_of_one_vertex() {
        let mut mesh = loop_mesh(false);
        let faces_before = face_tuples(&mesh);
        let owner = encode(&mut mesh, &[0, 1, 0, 1]).unwrap();
        assert_eq!(mesh.vertices.positions.len(), 4);
        assert_eq!(owner, [0, 0, 2, 2]);
        // A's loops first (topological key met at vertex 0), each pair in
        // increasing uv order.
        assert_eq!(mesh.vertices.positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(mesh.vertices.positions[1], [1.0, 2.0, 3.0]);
        assert_eq!(mesh.vertices.positions[2], [4.0, 5.0, 6.0]);
        assert_eq!(mesh.vertices.positions[3], [4.0, 5.0, 6.0]);
        assert!(mesh.vertices.uvs[0][0][0] < mesh.vertices.uvs[0][1][0]);
        assert!(mesh.vertices.uvs[0][2][0] < mesh.vertices.uvs[0][3][0]);
        // The decoded partition matches what was passed in: A's loops and
        // B's loops each sit under one owner.
        assert_eq!(decode(&mesh), owner);
        assert_eq!(face_tuples(&mesh), faces_before);
    }

    #[test]
    fn encode_collapses_identical_loops() {
        let mut mesh = loop_mesh(true);
        let faces_before = face_tuples(&mesh);
        let owner = encode(&mut mesh, &[0, 1, 0, 1]).unwrap();
        assert_eq!(mesh.vertices.positions.len(), 3);
        // A has a single loop left, owning itself.
        assert_eq!(owner, [0, 1, 1]);
        assert_eq!(face_tuples(&mesh), faces_before);
    }

    #[test]
    fn distinct_vertices_with_equal_keys_stay_distinct() {
        // Two owners sharing a position: the decreasing first-loop order
        // keeps them from reading as one vertex.
        let mut mesh = Mesh {
            vertices: MeshVertices {
                positions: vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]],
                normals: Some(vec![[0.0, 0.0, 1.0, 0.0]; 2]),
                uvs: vec![vec![[0.0, 0.0], [1.0, 0.0]]],
                uv_high_precision: vec![true],
                ..MeshVertices::default()
            },
            faces: vec![[0, 1, 0]],
            bone_group: Vec::new(),
            material: 0,
            alpha_flags: 0,
            shadow_flags: 0,
            has_antiblur_meshes: false,
            is_antiblur_mesh: false,
            custom_bounding_box: None,
        };
        let owner = encode(&mut mesh, &[0, 1]).unwrap();
        // The vertex with the larger loop encoding comes first.
        assert_eq!(mesh.vertices.uvs[0][0], [1.0, 0.0]);
        assert_eq!(mesh.vertices.uvs[0][1], [0.0, 0.0]);
        assert_eq!(owner, [0, 1]);
        // decode still separates them into two vertices.
        let decoded = decode(&mesh);
        assert_eq!(decoded, [0, 1]);
        assert_eq!(partition(&decoded).len(), 2);
    }

    #[test]
    fn model_level_round_trip() {
        let mut model = load(ORAL);
        let owners = decode_model(&model);
        let new_owners = encode_model(&mut model, &owners).unwrap();
        assert!(model.extensions.vertex_loop_preservation);
        assert_eq!(
            new_owners
                .iter()
                .map(|owner| partition(owner))
                .collect::<Vec<_>>(),
            owners
                .iter()
                .map(|owner| partition(owner))
                .collect::<Vec<_>>()
        );
        let again = Model::from_file(&model.to_file().unwrap()).unwrap();
        assert!(again.extensions.vertex_loop_preservation);
        assert_eq!(
            decode_model(&again)
                .iter()
                .map(|owner| partition(owner))
                .collect::<Vec<_>>(),
            owners
                .iter()
                .map(|owner| partition(owner))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn owner_errors() {
        let mut mesh = loop_mesh(false);
        assert!(matches!(
            encode(&mut mesh, &[0, 1, 0]),
            Err(FmdlError::VertexMismatch(_))
        ));
        assert!(matches!(
            encode(&mut mesh, &[0, 1, 0, 9]),
            Err(FmdlError::VertexMismatch(_))
        ));
    }
}
