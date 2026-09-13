use std::collections::{BTreeSet, HashSet};

use super::*;
use crate::format::FmdlFile;
use crate::model::{Bone, BoundingBox, Extensions, MaterialInstance};

const HIGHNECK: &[u8] = include_bytes!("../../../tests/fixtures/konami_highneck.fmdl");
const MOUTH: &[u8] = include_bytes!("../../../tests/fixtures/konami_mouth.fmdl");
const AU_LOW: &[u8] = include_bytes!("../../../tests/fixtures/konami_au_Low_parts.fmdl");
const ORAL: &[u8] = include_bytes!("../../../tests/fixtures/addon_oral.fmdl");
const PLACEHOLDER: &[u8] = include_bytes!("../../../tests/fixtures/addon_placeholder.fmdl");
const FIXTURES: [&[u8]; 5] = [HIGHNECK, MOUTH, AU_LOW, ORAL, PLACEHOLDER];

fn fixture_model(bytes: &[u8]) -> Model {
    Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
}

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

fn material() -> MaterialInstance {
    MaterialInstance {
        name: "Material".to_owned(),
        shader: "fox3ddf_blin".to_owned(),
        technique: "fox3DDF_Blin".to_owned(),
        textures: Vec::new(),
        parameters: Vec::new(),
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
        colors: None,
        uvs: vec![Vec::new()],
        uv_high_precision: vec![true],
        bone_weights: (bones > 0).then(Vec::new),
        bone_indices: (bones > 0).then(Vec::new),
    };
    for y in 0..height {
        for x in 0..width {
            vertices.positions.push([x as f32, y as f32, 0.0]);
            vertices
                .normals
                .as_mut()
                .unwrap()
                .push([0.0, 0.0, 1.0, 0.0]);
            vertices.uvs[0].push([x as f32 / width as f32, y as f32 / height as f32]);
            if bones > 0 {
                let bone = (x * bones / width) as u8;
                vertices.bone_weights.as_mut().unwrap().push([255, 0, 0, 0]);
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
    Mesh {
        vertices,
        faces,
        bone_group: (0..bones).collect(),
        material: 0,
        alpha_flags: 0,
        shadow_flags: 0,
        has_antiblur_meshes: false,
        is_antiblur_mesh: false,
        custom_bounding_box: None,
    }
}

/// One mesh in one group in a model with `bones` chain-parented bones.
fn grid_model(mesh: Mesh, bones: usize) -> Model {
    Model {
        bones: (0..bones)
            .map(|index| bone(&format!("bone{index}"), index.checked_sub(1)))
            .collect(),
        materials: vec![material()],
        meshes: vec![mesh],
        mesh_groups: vec![MeshGroup {
            name: "group".to_owned(),
            parent: None,
            meshes: vec![0],
            bounding_box: Some(BoundingBox {
                max: [10.0, 10.0, 10.0, 1.0],
                min: [-10.0, -10.0, -10.0, 1.0],
            }),
            visible: true,
            split_mesh_group: false,
        }],
        extensions: Extensions::default(),
        bone_matrices: Some(Vec::new()),
    }
}

/// A vertex resolved to the values a face sees: position, normal, uv
/// maps and the model-level bone mapping (bits, so tuples order).
type VertexTuple = ([u32; 3], Option<[u32; 4]>, Vec<[u32; 2]>, BoneMapping);

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
        bone_mapping(mesh, index).unwrap(),
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
    mesh.bone_group
        .iter()
        .copied()
        .filter(|bone| used.contains(bone))
        .collect()
}

// S1
#[test]
fn needs_splitting_limits() {
    for bytes in FIXTURES {
        let model = fixture_model(bytes);
        for mesh in &model.meshes {
            assert!(!needs_splitting(mesh));
        }
    }
    assert!(needs_splitting(&grid(60, 20, 40)));
    assert!(needs_splitting(&grid(300, 300, 0)));
    assert!(needs_splitting(&grid(150, 150, 0)));
}

// S2
#[test]
fn split_bones() {
    let source = grid(60, 20, 40);
    let source_faces = face_tuples(&source);
    let mut model = grid_model(source, 40);
    assert!(encode(&mut model, None).unwrap());

    components_under_limits(&model);
    assert!(model.meshes.len() > 1);
    let union: BTreeSet<[VertexTuple; 3]> = model.meshes.iter().flat_map(face_tuples).collect();
    assert_eq!(union, source_faces);

    assert_eq!(model.mesh_groups.len(), 2);
    assert_eq!(model.mesh_groups[0].name, "group");
    assert!(model.mesh_groups[0].meshes.is_empty());
    let split = &model.mesh_groups[1];
    assert_eq!(split.name, "split-mesh");
    assert_eq!(split.parent, Some(0));
    assert!(split.split_mesh_group);
    assert_eq!(split.bounding_box, model.mesh_groups[0].bounding_box);
    assert_eq!(split.visible, model.mesh_groups[0].visible);
    assert_eq!(
        split.meshes,
        (0..model.meshes.len()).collect::<Vec<usize>>()
    );
    assert!(model.extensions.mesh_splitting);
}

// S3
#[test]
fn split_vertices() {
    let source = grid(300, 300, 0);
    let source_faces = face_tuples(&source);
    let mut model = grid_model(source, 0);
    assert!(encode(&mut model, None).unwrap());

    components_under_limits(&model);
    assert!(model.meshes.len() > 1);
    let union: BTreeSet<[VertexTuple; 3]> = model.meshes.iter().flat_map(face_tuples).collect();
    assert_eq!(union, source_faces);
}

// S4
fn check_round_trip(mut model: Model, source: Mesh) {
    let source_faces = face_tuples(&source);
    let source_bones = used_bones(&source);
    assert!(encode(&mut model, None).unwrap());
    decode(&mut model).unwrap();

    assert_eq!(model.meshes.len(), 1);
    let combined = &model.meshes[0];
    assert_eq!(face_tuples(combined), source_faces);
    assert_eq!(combined.material, source.material);
    assert_eq!(combined.alpha_flags, source.alpha_flags);
    assert_eq!(combined.shadow_flags, source.shadow_flags);
    assert_eq!(combined.has_antiblur_meshes, source.has_antiblur_meshes);
    assert_eq!(combined.is_antiblur_mesh, source.is_antiblur_mesh);
    assert_eq!(combined.bone_group, source_bones);

    assert_eq!(model.mesh_groups.len(), 1);
    assert_eq!(model.mesh_groups[0].name, "group");
    assert_eq!(model.mesh_groups[0].meshes, vec![0]);
    assert!(!model.extensions.mesh_splitting);
}

#[test]
fn round_trip_bones() {
    let source = grid(60, 20, 40);
    let model = grid_model(source.clone(), 40);
    check_round_trip(model, source);
}

#[test]
fn round_trip_vertices() {
    let source = grid(300, 300, 0);
    let model = grid_model(source.clone(), 0);
    check_round_trip(model, source);
}

// S5: two byte-identical vertices, each referenced by faces that must
// land in different components (disjoint bone sets over the limit).
#[test]
fn duplicate_vertices() {
    let bones = 40;
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
            .push([0.0, 0.0, 1.0, 0.0]);
        mesh.vertices.uvs[0].push([position[0] / 200.0, position[1]]);
        mesh.vertices
            .bone_weights
            .as_mut()
            .unwrap()
            .push([255, 0, 0, 0]);
        mesh.vertices
            .bone_indices
            .as_mut()
            .unwrap()
            .push([bone, 0, 0, 0]);
    };
    // Vertices 0 and 1: identical in every stored byte.
    add(&mut mesh, [0.0, 0.0, 0.0], 0);
    add(&mut mesh, [0.0, 0.0, 0.0], 0);
    // Cluster A: 16 vertices on bones 0..16, faces through vertex 0.
    for i in 0..16 {
        add(&mut mesh, [10.0 + i as f32, 0.0, 0.0], i);
    }
    for i in 0..15usize {
        mesh.faces.push([0, (2 + i) as u16, (2 + i + 1) as u16]);
    }
    // Cluster B: 20 vertices on bones 20..40, faces through vertex 1.
    for i in 0..20 {
        add(&mut mesh, [100.0 + i as f32, 1.0, 0.0], 20 + i);
    }
    for i in 0..19usize {
        mesh.faces.push([1, (18 + i) as u16, (18 + i + 1) as u16]);
    }
    let source_faces = face_tuples(&mesh);
    let source_count = mesh.vertices.positions.len();
    let mut model = grid_model(mesh, bones);
    assert!(encode(&mut model, None).unwrap());

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

// S6: a small mesh alongside an oversized one.
#[test]
fn untouched_mesh() {
    let small = grid(8, 8, 2);
    let source = grid(60, 20, 40);
    let source_faces = face_tuples(&source);
    let source_bones = used_bones(&source);
    let mut model = Model {
        bones: (0..40usize)
            .map(|index| bone(&format!("bone{index}"), index.checked_sub(1)))
            .collect(),
        materials: vec![material()],
        meshes: vec![small.clone(), source],
        mesh_groups: vec![MeshGroup {
            name: "group".to_owned(),
            parent: None,
            meshes: vec![0, 1],
            bounding_box: Some(BoundingBox {
                max: [10.0, 10.0, 10.0, 1.0],
                min: [-10.0, -10.0, -10.0, 1.0],
            }),
            visible: true,
            split_mesh_group: false,
        }],
        extensions: Extensions::default(),
        bone_matrices: None,
    };

    assert!(encode(&mut model, None).unwrap());
    assert_eq!(model.meshes[0], small);
    assert_eq!(model.mesh_groups[0].meshes, vec![0]);
    assert!(model.mesh_groups[1].split_mesh_group);

    decode(&mut model).unwrap();
    assert_eq!(model.meshes.len(), 2);
    assert_eq!(model.meshes[0], small);
    let combined = &model.meshes[1];
    // Vertex order is not restored by construction; compare the face
    // tuple set, vertex count, and every non-order field.
    assert_eq!(combined.vertices.positions.len(), 60 * 20);
    assert_eq!(face_tuples(combined), source_faces);
    assert_eq!(combined.bone_group, source_bones);
    assert_eq!(combined.material, 0);
    assert_eq!(combined.alpha_flags, 0);
    assert_eq!(combined.shadow_flags, 0);
    assert_eq!(model.mesh_groups.len(), 1);
    assert_eq!(model.mesh_groups[0].meshes, vec![0, 1]);
    assert!(!model.extensions.mesh_splitting);
}

// S7
#[test]
fn file_round_trip() {
    let mut model = grid_model(grid(60, 20, 40), 40);
    assert!(encode(&mut model, None).unwrap());
    let written = model.to_file().unwrap();
    let again = Model::from_file(&FmdlFile::read(&written.write()).unwrap()).unwrap();
    assert_eq!(again, model);
}

// S8
#[test]
fn effective_parents_chain() {
    let model = Model {
        bones: vec![
            bone("dsk_hip", None),
            bone("sk_belly", Some(0)),
            bone("sk_chest", Some(1)),
        ],
        materials: Vec::new(),
        meshes: Vec::new(),
        mesh_groups: Vec::new(),
        extensions: Extensions::default(),
        bone_matrices: None,
    };
    let parents = effective_parents(&model);
    assert_eq!(parents[2], None); // sk_chest is the root
    assert_eq!(parents[1], Some(2)); // sk_belly's parent is the chest
    assert_eq!(parents[0], Some(1)); // dsk_hip's parent is the belly

    let highneck = fixture_model(HIGHNECK);
    let parents = effective_parents(&highneck);
    let expected: Vec<Option<usize>> = highneck.bones.iter().map(|bone| bone.parent).collect();
    assert_eq!(parents, expected);
}

// S9
#[test]
fn decode_errors() {
    let mut model = grid_model(grid(4, 4, 0), 0);
    model.mesh_groups.push(MeshGroup {
        name: "split-mesh".to_owned(),
        parent: None,
        meshes: vec![0],
        bounding_box: None,
        visible: true,
        split_mesh_group: true,
    });
    model.mesh_groups[0].meshes.clear();
    assert!(matches!(
        decode(&mut model),
        Err(FmdlError::BadMeshGroupAssignment(_))
    ));

    let mut model = grid_model(grid(4, 4, 0), 0);
    model.mesh_groups.push(MeshGroup {
        name: "split-mesh".to_owned(),
        parent: Some(0),
        meshes: vec![0],
        bounding_box: None,
        visible: true,
        split_mesh_group: true,
    });
    model.mesh_groups[0].meshes.clear();
    model.mesh_groups.push(MeshGroup {
        name: "child".to_owned(),
        parent: Some(1),
        meshes: Vec::new(),
        bounding_box: None,
        visible: true,
        split_mesh_group: false,
    });
    assert!(matches!(
        decode(&mut model),
        Err(FmdlError::BadMeshGroupAssignment(_))
    ));
}
