use std::collections::{BTreeSet, HashSet};

use super::*;
use crate::format::{Bone, BoundingBox, LodRecord, PreFoxModel};

fn bone(name: &str) -> Bone {
    Bone {
        name: name.to_owned(),
        matrix: [
            1.0, 0.0, 0.0, 0.0, // row 0
            0.0, 1.0, 0.0, 0.0, // row 1
            0.0, 0.0, 1.0, 0.0, // row 2
        ],
    }
}

/// A width x height grid: vertex `y * width + x` at `[x, y, 0]`,
/// weighted to bone `x * bones / width`, two faces per cell where
/// every index fits in u16 (larger grids leave the rest loose).
fn grid(width: usize, height: usize, bones: usize) -> Mesh {
    let mut vertices = MeshVertices {
        positions: Vec::new(),
        normals: Some(Vec::new()),
        tangents: None,
        bitangents: None,
        colors: None,
        uvs: vec![Vec::new()],
        bone_indices: (bones > 0).then(Vec::new),
        bone_weights: (bones > 0).then(Vec::new),
        bone_weight_width: 4,
    };
    for y in 0..height {
        for x in 0..width {
            vertices.positions.push([x as f32, y as f32, 0.0]);
            vertices.normals.as_mut().unwrap().push([0.0, 0.0, 1.0]);
            vertices.uvs[0].push([x as f32 / width as f32, y as f32 / height as f32]);
            if bones > 0 {
                let bone = (x * bones / width) as u8;
                vertices
                    .bone_weights
                    .as_mut()
                    .unwrap()
                    .push([1.0, 0.0, 0.0, 0.0]);
                vertices
                    .bone_indices
                    .as_mut()
                    .unwrap()
                    .push([bone, 0, 0, 0]);
            }
        }
    }
    let mut faces = Vec::new();
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let v00 = y * width + x;
            let v10 = v00 + 1;
            let v01 = v00 + width;
            let v11 = v01 + 1;
            if v11 <= usize::from(u16::MAX) {
                faces.push([v00 as u16, v10 as u16, v11 as u16]);
                faces.push([v00 as u16, v11 as u16, v01 as u16]);
            }
        }
    }
    let bounds = BoundingBox::of(&vertices.positions);
    Mesh {
        name: None,
        extension_headers: Vec::new(),
        tags: Vec::new(),
        vertices,
        faces,
        lower_lods: Vec::new(),
        bone_group: (0..bones).collect(),
        material: 0,
        bounds,
        order: 0,
        editor_data: Vec::new(),
    }
}

/// One mesh in a model with `bones` bones (all roots).
fn grid_model(mesh: Mesh, bones: usize) -> Model {
    Model {
        flags: 0,
        bones: (0..bones)
            .map(|index| bone(&format!("bone{index}")))
            .collect(),
        materials: vec!["Material".to_owned()],
        meshes: vec![mesh],
        extension_headers: Vec::new(),
        bounds: BoundingBox::of(&[]),
        lod: LodRecord::for_levels(0),
    }
}

/// A vertex resolved to the values a face sees: position, normal, uv
/// maps and the model-level bone mapping (bits, so tuples order).
type VertexTuple = ([u32; 3], Option<[u32; 3]>, Vec<[u32; 2]>, Vec<(usize, u32)>);

fn vertex_tuple(mesh: &Mesh, index: usize) -> VertexTuple {
    (
        mesh.vertices.positions[index].map(f32::to_bits),
        mesh.vertices
            .normals
            .as_ref()
            .map(|normals| normals[index].map(f32::to_bits)),
        mesh.vertices
            .uvs
            .iter()
            .map(|map| map[index].map(f32::to_bits))
            .collect(),
        bone_mapping(mesh, index)
            .unwrap()
            .iter()
            .map(|&(bone, weight)| (bone, weight.to_bits()))
            .collect(),
    )
}

fn face_tuples(mesh: &Mesh) -> BTreeSet<[VertexTuple; 3]> {
    mesh.faces
        .iter()
        .map(|face| face.map(|index| vertex_tuple(mesh, usize::from(index))))
        .collect()
}

fn components_under_limits(model: &Model) {
    for mesh in &model.meshes {
        assert!(mesh.bone_group.len() <= BONE_LIMIT_SOFT);
        assert!(mesh.vertices.positions.len() <= VERTEX_LIMIT_SOFT);
        assert!(mesh.faces.len() <= FACE_LIMIT_SOFT);
    }
}

/// The source's used bones in source bone-group order.
fn used_bones(mesh: &Mesh) -> Vec<usize> {
    let mut used = HashSet::new();
    for index in 0..mesh.vertices.positions.len() {
        for (bone, _) in bone_mapping(mesh, index).unwrap() {
            used.insert(bone);
        }
    }
    let mut bones: Vec<usize> = used.iter().copied().collect();
    bones.sort_unstable();
    bones
}

fn split_headers(mesh: &Mesh) -> Vec<String> {
    mesh.extension_headers
        .iter()
        .filter(|header| split_group_key(header).is_some())
        .cloned()
        .collect()
}

// 1
#[test]
fn needs_splitting_limits() {
    let mut mesh = grid(8, 8, 0);
    assert!(!needs_splitting(&mesh));
    mesh.bone_group = (0..BONE_LIMIT_HARD).collect();
    assert!(!needs_splitting(&mesh));
    mesh.bone_group = (0..BONE_LIMIT_HARD + 1).collect();
    assert!(needs_splitting(&mesh));
    mesh.bone_group = Vec::new();

    mesh.vertices.positions = vec![[0.0; 3]; VERTEX_LIMIT_HARD];
    assert!(!needs_splitting(&mesh));
    mesh.vertices.positions.push([0.0; 3]);
    assert!(needs_splitting(&mesh));
    mesh.vertices.positions = vec![[0.0; 3]; 4];

    mesh.faces = vec![[0, 1, 2]; FACE_LIMIT_HARD];
    assert!(!needs_splitting(&mesh));
    mesh.faces.push([0, 1, 2]);
    assert!(needs_splitting(&mesh));
}

// 2
#[test]
fn effective_parents_chain() {
    let model = Model {
        flags: 0,
        bones: vec![bone("dsk_hip"), bone("sk_belly"), bone("sk_chest")],
        materials: Vec::new(),
        meshes: Vec::new(),
        extension_headers: Vec::new(),
        bounds: BoundingBox::of(&[]),
        lod: LodRecord::for_levels(0),
    };
    let parents = effective_parents(&model, &[None, Some(0), Some(1)]);
    assert_eq!(parents[2], None); // sk_chest is the root
    assert_eq!(parents[1], Some(2)); // sk_belly's parent is the chest
    assert_eq!(parents[0], Some(1)); // dsk_hip's parent is the belly

    // A parent cycle is cut.
    let cycle = effective_parents(&model, &[Some(1), Some(0), None]);
    assert_eq!(cycle[1], None);

    // All roots stays all roots.
    assert_eq!(
        effective_parents(&model, &[None, None, None]),
        [None, None, None]
    );
}

// 3
#[test]
fn untouched_mesh() {
    let mut model = grid_model(grid(8, 8, 2), 2);
    let before = model.clone();
    assert!(!encode(&mut model, &[None, None]).unwrap());
    assert_eq!(model, before);
}

// 4
#[test]
fn split_bones() {
    let source = grid(70, 20, 70);
    let source_faces = face_tuples(&source);
    let mut model = grid_model(source, 70);
    let parents: Vec<Option<usize>> = vec![None; 70];
    assert!(encode(&mut model, &parents).unwrap());

    components_under_limits(&model);
    assert!(model.meshes.len() > 1);
    let union: BTreeSet<[VertexTuple; 3]> = model.meshes.iter().flat_map(face_tuples).collect();
    assert_eq!(union, source_faces);

    decode(&mut model).unwrap();
    assert_eq!(model.meshes.len(), 1);
    let combined = &model.meshes[0];
    assert_eq!(combined.vertices.positions.len(), 70 * 20);
    assert_eq!(face_tuples(combined), source_faces);
}

// 5
#[test]
fn split_vertices() {
    let source = grid(300, 300, 0);
    let source_faces = face_tuples(&source);
    let mut model = grid_model(source, 0);
    assert!(encode(&mut model, &[]).unwrap());

    components_under_limits(&model);
    assert!(model.meshes.len() > 1);
    let union: BTreeSet<[VertexTuple; 3]> = model.meshes.iter().flat_map(face_tuples).collect();
    assert_eq!(union, source_faces);
}

// 6
fn check_round_trip(mut model: Model, source: Mesh, parents: &[Option<usize>]) {
    let source_faces = face_tuples(&source);
    let source_bones = used_bones(&source);
    assert!(encode(&mut model, parents).unwrap());
    decode(&mut model).unwrap();

    assert_eq!(model.meshes.len(), 1);
    let combined = &model.meshes[0];
    assert_eq!(face_tuples(combined), source_faces);
    assert_eq!(combined.material, source.material);
    assert_eq!(combined.name, source.name);
    assert_eq!(combined.tags, source.tags);
    assert_eq!(combined.order, source.order);
    assert_eq!(combined.editor_data, source.editor_data);
    assert_eq!(combined.bone_group, source_bones);
    assert!(split_headers(combined).is_empty());
}

#[test]
fn round_trip_bones() {
    let source = grid(70, 20, 70);
    let model = grid_model(source.clone(), 70);
    check_round_trip(model, source, &vec![None; 70]);
}

#[test]
fn round_trip_vertices() {
    let source = grid(300, 300, 0);
    let model = grid_model(source.clone(), 0);
    check_round_trip(model, source, &[]);
}

// 7: two byte-identical vertices, each referenced by faces that must
// land in different components (disjoint bone sets over the limit).
#[test]
fn duplicate_vertices() {
    let bones = 70;
    let mut mesh = grid(1, 1, bones);
    mesh.vertices.positions.clear();
    mesh.vertices.normals.as_mut().unwrap().clear();
    mesh.vertices.uvs[0].clear();
    mesh.vertices.bone_weights.as_mut().unwrap().clear();
    mesh.vertices.bone_indices.as_mut().unwrap().clear();
    mesh.faces.clear();
    let add = |mesh: &mut Mesh, position: [f32; 3], bone: u8| {
        mesh.vertices.positions.push(position);
        mesh.vertices
            .normals
            .as_mut()
            .unwrap()
            .push([0.0, 0.0, 1.0]);
        mesh.vertices.uvs[0].push([position[0] / 200.0, position[1]]);
        mesh.vertices
            .bone_weights
            .as_mut()
            .unwrap()
            .push([1.0, 0.0, 0.0, 0.0]);
        mesh.vertices
            .bone_indices
            .as_mut()
            .unwrap()
            .push([bone, 0, 0, 0]);
    };
    // Vertices 0 and 1: identical in every stored byte.
    add(&mut mesh, [0.0, 0.0, 0.0], 0);
    add(&mut mesh, [0.0, 0.0, 0.0], 0);
    // Cluster A: 45 vertices on bones 0..45, faces through vertex 0.
    for i in 0..45 {
        add(&mut mesh, [10.0 + i as f32, 0.0, 0.0], i);
    }
    for i in 0..44usize {
        mesh.faces.push([0, (2 + i) as u16, (2 + i + 1) as u16]);
    }
    // Cluster B: 25 vertices on bones 45..70, faces through vertex 1.
    for i in 0..25 {
        add(&mut mesh, [100.0 + i as f32, 1.0, 0.0], 45 + i);
    }
    for i in 0..24usize {
        mesh.faces.push([1, (47 + i) as u16, (47 + i + 1) as u16]);
    }
    let source_faces = face_tuples(&mesh);
    let source_count = mesh.vertices.positions.len();
    let mut model = grid_model(mesh, bones);
    let parents: Vec<Option<usize>> = vec![None; bones];
    assert!(encode(&mut model, &parents).unwrap());

    // The duplicate key: position [0,0,0], uv [0,0].
    let is_dup = |mesh: &Mesh, index: usize| {
        mesh.vertices.positions[index] == [0.0, 0.0, 0.0]
            && mesh.vertices.uvs[0][index] == [0.0, 0.0]
    };
    let mut components_with_dups = 0;
    for component in &model.meshes {
        let dups: Vec<usize> = (0..component.vertices.positions.len())
            .filter(|&index| is_dup(component, index))
            .collect();
        if !dups.is_empty() {
            assert_eq!(dups.len(), 2);
            assert!(dups[0] < dups[1]);
            components_with_dups += 1;
        }
    }
    assert!(components_with_dups >= 2);

    decode(&mut model).unwrap();
    assert_eq!(model.meshes.len(), 1);
    let combined = &model.meshes[0];
    let dups: Vec<usize> = (0..combined.vertices.positions.len())
        .filter(|&index| is_dup(combined, index))
        .collect();
    assert_eq!(dups.len(), 2);
    assert_eq!(combined.vertices.positions.len(), source_count);
    assert_eq!(face_tuples(combined), source_faces);
}

// 8
#[test]
fn decode_errors() {
    // Two components of one group disagreeing on material.
    let mut model = grid_model(grid(4, 4, 0), 0);
    model.materials.push("Other".to_owned());
    let mut second = model.meshes[0].clone();
    second.material = 1;
    model.meshes[0]
        .extension_headers
        .push("Split-Mesh: 1".to_owned());
    second.extension_headers.push("Split-Mesh: 1".to_owned());
    model.meshes.push(second);
    assert!(matches!(
        decode(&mut model),
        Err(ModelError::InvalidModel(_))
    ));

    // A component with a face index past its vertices.
    let mut model = grid_model(grid(4, 4, 0), 0);
    let mut second = model.meshes[0].clone();
    model.meshes[0]
        .extension_headers
        .push("Split-Mesh: 1".to_owned());
    second.extension_headers.push("Split-Mesh: 1".to_owned());
    second.faces[0][0] = u16::MAX;
    model.meshes.push(second);
    assert!(matches!(
        decode(&mut model),
        Err(ModelError::BadReference { what: "vertex", .. })
    ));

    // Components disagreeing on bone weight width.
    let mut model = grid_model(grid(4, 4, 2), 2);
    let mut second = model.meshes[0].clone();
    model.meshes[0]
        .extension_headers
        .push("Split-Mesh: 1".to_owned());
    second.vertices.bone_weight_width = 3;
    second.extension_headers.push("Split-Mesh: 1".to_owned());
    model.meshes.push(second);
    assert!(matches!(
        decode(&mut model),
        Err(ModelError::InvalidModel(message)) if message == "split components disagree on bone weight width"
    ));
}

// 9
#[test]
fn lods_refuse_to_split() {
    let mut mesh = grid(150, 150, 0);
    mesh.lower_lods = vec![vec![[0, 1, 2]]];
    let mut model = grid_model(mesh, 0);
    assert!(matches!(
        encode(&mut model, &[]),
        Err(ModelError::InvalidModel(message)) if message == "cannot split a mesh with LOD levels"
    ));
}

// 10
#[test]
fn file_round_trip() {
    let mut model = grid_model(grid(70, 20, 70), 70);
    let parents: Vec<Option<usize>> = vec![None; 70];
    assert!(encode(&mut model, &parents).unwrap());
    let count = model.meshes.len();
    let written = model.to_file().unwrap().write().unwrap();
    let mut again = Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap();
    assert_eq!(again.meshes.len(), count);
    for mesh in &again.meshes {
        assert_eq!(split_headers(mesh), ["Split-Mesh: 1".to_owned()]);
    }
    decode(&mut again).unwrap();
    assert_eq!(again.meshes.len(), 1);
}

// 11
#[test]
fn two_sources_number_from_one() {
    let mut model = Model {
        flags: 0,
        bones: (0..70).map(|index| bone(&format!("bone{index}"))).collect(),
        materials: vec!["Material".to_owned()],
        meshes: vec![grid(70, 20, 70), grid(8, 8, 2), grid(70, 20, 70)],
        extension_headers: Vec::new(),
        bounds: BoundingBox::of(&[]),
        lod: LodRecord::for_levels(0),
    };
    let parents: Vec<Option<usize>> = vec![None; 70];
    assert!(encode(&mut model, &parents).unwrap());

    // The fitting mesh sits between the two split groups.
    let fitting = model
        .meshes
        .iter()
        .position(|mesh| mesh.vertices.positions.len() == 8 * 8)
        .unwrap();
    for mesh in &model.meshes[..fitting] {
        assert_eq!(split_headers(mesh), ["Split-Mesh: 1".to_owned()]);
    }
    assert!(model.meshes[..fitting].len() > 1);
    for mesh in &model.meshes[fitting + 1..] {
        assert_eq!(split_headers(mesh), ["Split-Mesh: 2".to_owned()]);
    }
    assert!(model.meshes.len() > fitting + 2);
}
