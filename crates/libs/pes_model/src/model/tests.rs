use super::*;
use crate::format::fixtures::*;
use crate::format::{DatumType, LodRecord, VertexField};

#[test]
fn every_fixture_round_trips_semantically() {
    for bytes in ALL {
        let model = Model::from_file(&PreFoxModel::read(bytes).unwrap()).unwrap();
        let file = model.to_file().unwrap();
        assert_eq!(Model::from_file(&file).unwrap(), model);
        // The full trip through bytes.
        let written = file.write().unwrap();
        assert_eq!(
            Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
            model
        );
    }
}

#[test]
fn content() {
    let cap = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    assert_eq!(cap.bones.len(), 4);
    assert_eq!(cap.materials, ["modD_cap_phone"]);
    assert_eq!(cap.meshes.len(), 1);
    let mesh = &cap.meshes[0];
    assert_eq!(mesh.name, None);
    assert!(mesh.extension_headers.is_empty());
    assert_eq!(mesh.tags, [(1, "DCaptainmark".to_owned())]);
    assert_eq!(mesh.bone_group, [0, 1, 3, 2]);
    assert_eq!(mesh.material, 0);
    assert_eq!(mesh.faces.len(), 78);
    assert_eq!(mesh.faces[0], [0, 4, 1]);
    assert!(mesh.lower_lods.is_empty());
    assert_eq!(mesh.vertices.len(), 56);
    assert_eq!(mesh.order, 0);
    assert!(mesh.editor_data.is_empty());
    assert!(cap.extension_headers.is_empty());
    assert_eq!(cap.lod, LodRecord::for_levels(0));

    let cardhead = Model::from_file(&PreFoxModel::read(CARDHEAD).unwrap()).unwrap();
    let mesh = &cardhead.meshes[0];
    assert_eq!(mesh.name, Some("card".to_owned()));
    assert_eq!(mesh.extension_headers, ["vertex-loop-preservation"]);
    assert!(mesh.tags.is_empty());
    assert_eq!(cardhead.extension_headers, ["Skeleton-Type: Simplified"]);
    assert_eq!(mesh.bone_group, [0]);

    let glasses = Model::from_file(&PreFoxModel::read(GLASSES).unwrap()).unwrap();
    assert_eq!(glasses.meshes.len(), 2);
    assert_eq!(glasses.materials, ["glasses_02T", "glasses_02C"]);
    for (index, mesh) in glasses.meshes.iter().enumerate() {
        assert_eq!(
            mesh.tags,
            [(1, "glasses_02".to_owned()), (10, "glasses_02".to_owned())]
        );
        assert_eq!(mesh.bone_group, [0]);
        assert_eq!(mesh.material, index);
    }

    let collar = Model::from_file(&PreFoxModel::read(COLLAR).unwrap()).unwrap();
    let mesh = &collar.meshes[0];
    assert_eq!(mesh.faces.len(), 4080);
    assert_eq!(
        mesh.lower_lods
            .iter()
            .map(|level| level.len())
            .collect::<Vec<_>>(),
        [3090, 2281, 1626, 1093, 662]
    );
    assert_eq!(collar.lod, LodRecord::for_levels(6));
    assert_eq!(
        mesh.tags,
        [
            (1, "prt".to_owned()),
            (2, "DSpecularS".to_owned()),
            (7, "DNormalS".to_owned())
        ]
    );

    let flag = Model::from_file(&PreFoxModel::read(FLAG).unwrap()).unwrap();
    assert!(flag.bones.is_empty());
    assert!(flag.meshes[0].bone_group.is_empty());
    assert!(flag.meshes[0].vertices.bone_indices.is_none());

    let shadow = Model::from_file(&PreFoxModel::read(SHADOW).unwrap()).unwrap();
    assert_eq!(shadow.meshes[0].order, 0);
    assert!(shadow.meshes[0].editor_data.is_empty());
    let file = shadow.to_file().unwrap();
    assert_eq!(file.geometries[0].order, Some(0));
    assert_eq!(file.meshes[0].editor_data, Some(vec![]));
}

#[test]
fn helpers() {
    assert_eq!(
        LodRecord::for_levels(6),
        LodRecord {
            level_count: 6,
            parameters: [0.0625, 4.0, 0.3]
        }
    );
    assert_eq!(LodRecord::for_levels(0).parameters, [0.0625, 4.0, 0.0]);
    assert_eq!(
        BoundingBox::of(&[[1.0, 2.0, 3.0], [-1.0, 5.0, 0.0]]),
        BoundingBox {
            min: [-1.0, 2.0, 0.0, 0.0],
            max: [1.0, 5.0, 3.0, 0.0]
        }
    );
    assert_eq!(
        BoundingBox::of(&[]),
        BoundingBox {
            min: [0.0; 4],
            max: [0.0; 4]
        }
    );
    // The stored box contains the vertices; Konami's may be looser.
    // Community writers repoint geometry and leave the old box, which
    // the reader carries as read.
    for bytes in ALL.iter().filter(|bytes| !COMMUNITY.contains(bytes)) {
        let model = Model::from_file(&PreFoxModel::read(bytes).unwrap()).unwrap();
        for mesh in &model.meshes {
            let around = BoundingBox::of(&mesh.vertices.positions);
            for axis in 0..3 {
                assert!(around.min[axis] >= mesh.bounds.min[axis] - 1e-4);
                assert!(around.max[axis] <= mesh.bounds.max[axis] + 1e-4);
            }
        }
    }
}

#[test]
fn written_layout_facts() {
    let cap = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    let file = cap.to_file().unwrap();
    assert_eq!(file.annotation_strings, ["DCaptainmark"]);
    assert_eq!(file.annotation_records, [[0, 0, 0, 2, 2, 2, 0]]);
    assert_eq!(
        file.meshes[0]
            .annotations
            .iter()
            .map(|a| (a.string, a.record, a.unknown, a.kind))
            .collect::<Vec<_>>(),
        [(0, Some(0), 7, 1)]
    );

    let cardhead = Model::from_file(&PreFoxModel::read(CARDHEAD).unwrap()).unwrap();
    let file = cardhead.to_file().unwrap();
    assert_eq!(
        file.annotation_strings,
        [
            "card",
            "vertex-loop-preservation",
            "Skeleton-Type: Simplified"
        ]
    );
    assert!(file.annotation_records.is_empty());
    assert_eq!(
        file.meshes[0]
            .annotations
            .iter()
            .map(|a| (a.string, a.record, a.unknown, a.kind))
            .collect::<Vec<_>>(),
        [(0, None, 7, 128), (1, None, 7, 129)]
    );

    let glasses = Model::from_file(&PreFoxModel::read(GLASSES).unwrap()).unwrap();
    let file = glasses.to_file().unwrap();
    assert_eq!(file.annotation_strings, ["glasses_02"]);
    assert_eq!(file.annotation_records.len(), 4);
}

#[test]
fn dangling_references_error() {
    let mut model = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    model.meshes[0].material = 3;
    assert!(matches!(
        model.to_file(),
        Err(ModelError::BadReference {
            what: "material",
            ..
        })
    ));

    let mut model = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    model.meshes[0].bone_group = vec![9];
    assert!(matches!(
        model.to_file(),
        Err(ModelError::BadReference { what: "bone", .. })
    ));

    let mut model = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    model.meshes[0].faces[0] = [0, 1, 999];
    assert!(matches!(
        model.to_file(),
        Err(ModelError::BadReference { what: "vertex", .. })
    ));

    // A second kind-128 annotation on one mesh.
    let mut file = PreFoxModel::read(CARDHEAD).unwrap();
    let first = file.meshes[0].annotations[0].clone();
    file.meshes[0].annotations.push(first);
    assert_eq!(
        Model::from_file(&file),
        Err(ModelError::InvalidModel("two mesh names"))
    );

    let mut file = PreFoxModel::read(CAP).unwrap();
    file.geometries[0].faces.indices[0] = 999;
    assert!(matches!(
        Model::from_file(&file),
        Err(ModelError::BadReference { what: "vertex", .. })
    ));
}

#[test]
fn community_tail_data_content() {
    // Offsets name data past the end of the last section in file order;
    // the typed layer resolves them against the whole file.
    let model = Model::from_file(&PreFoxModel::read(COMMUNITY_TAIL_DATA).unwrap()).unwrap();
    assert_eq!(model.meshes.len(), 1);
    let mesh = &model.meshes[0];
    assert_eq!(mesh.vertices.len(), 171);
    assert_eq!(mesh.faces.len(), 246);
    assert_eq!(mesh.faces[0], [0, 65, 1]);
    assert_eq!(mesh.faces[1], [39, 65, 0]);
    assert_eq!(
        mesh.vertices.positions[0].map(f32::to_bits),
        [0xb207_8325, 0x3fcc_cccd, 0x3d4c_ccce]
    );
    assert_eq!(
        mesh.tags,
        [(1, "nld0027_d".to_owned()), (10, "nld0027_d".to_owned())]
    );
    assert_eq!(model.materials, ["modD_phone"]);
    let written = model.to_file().unwrap().write().unwrap();
    assert_eq!(
        Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
        model
    );
}

#[test]
fn community_template_annotation_content() {
    // The annotation's constant -124/-84 pointers land off sections 2
    // and 3, so it is dropped and the unreferenced string becomes a
    // model-level extension header.
    let model =
        Model::from_file(&PreFoxModel::read(COMMUNITY_TEMPLATE_ANNOTATION).unwrap()).unwrap();
    assert_eq!(model.meshes.len(), 1);
    let mesh = &model.meshes[0];
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.faces.len(), 1);
    assert_eq!(mesh.name, None);
    assert!(mesh.tags.is_empty());
    assert!(mesh.extension_headers.is_empty());
    assert_eq!(model.extension_headers, ["\u{5}\u{14}\u{2}\u{5}"]);
    assert_eq!(model.materials, ["empty"]);
    let written = model.to_file().unwrap().write().unwrap();
    assert_eq!(
        Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
        model
    );
}

#[test]
fn community_empty_geometry_content() {
    // Every field and face offset points at or past the end of the
    // file; the zero-length reads all succeed.
    let model = Model::from_file(&PreFoxModel::read(COMMUNITY_EMPTY_GEOMETRY).unwrap()).unwrap();
    assert!(!model.meshes.is_empty());
    for mesh in &model.meshes {
        assert_eq!(mesh.vertices.len(), 0);
        assert!(mesh.faces.is_empty());
    }
    let written = model.to_file().unwrap().write().unwrap();
    assert_eq!(
        Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
        model
    );
}

#[test]
fn community_duplicate_field_content() {
    // Geometry 0 carries a second bone-weights descriptor inside the
    // first one's data; the first descriptor of a type is the field
    // and the later one is ignored, bytes untouched.
    let file = PreFoxModel::read(COMMUNITY_DUPLICATE_FIELD).unwrap();
    let fields = &file.geometries[0].vertex_fields;
    assert_eq!(fields.len(), 8);
    assert_eq!(
        fields
            .iter()
            .filter(|field| field.datum_type == DatumType::BoneWeights)
            .map(VertexField::count)
            .collect::<Vec<_>>(),
        [8, 4]
    );
    let model = Model::from_file(&file).unwrap();
    let mesh = &model.meshes[file
        .meshes
        .iter()
        .position(|mesh| mesh.geometry == 0)
        .unwrap()];
    assert_eq!(mesh.vertices.len(), 8);
    let weights = mesh.vertices.bone_weights.as_ref().unwrap();
    assert_eq!(weights.len(), 8);
    // The decoded weights are the first bone-weights descriptor's bytes.
    let first = fields
        .iter()
        .position(|field| field.datum_type == DatumType::BoneWeights)
        .unwrap();
    let (words, _) = fields[first].data.as_chunks::<4>();
    let expected: Vec<[f32; 4]> = words
        .chunks(4)
        .map(|quad| {
            let quad: [[u8; 4]; 4] = quad.try_into().unwrap();
            quad.map(f32::from_le_bytes)
        })
        .collect();
    assert_eq!(weights, &expected);
    let written = model.to_file().unwrap().write().unwrap();
    assert_eq!(
        Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
        model
    );
}

#[test]
fn community_loose_vertex_content() {
    // The exporter counted loose vertices it did not write: the
    // distinct indices number exactly the vertex count, so every index
    // is remapped to its rank among them.
    let file = PreFoxModel::read(COMMUNITY_LOOSE_VERTEX).unwrap();
    let raw = file.geometries[2].faces.level(0).unwrap();
    let g = (0u16..=1058)
        .find(|value| !raw.iter().flatten().any(|index| index == value))
        .unwrap();
    let at = |geometry: usize| {
        file.meshes
            .iter()
            .position(|mesh| mesh.geometry == geometry)
            .unwrap()
    };
    let model = Model::from_file(&file).unwrap();
    let mesh = &model.meshes[at(2)];
    assert_eq!(mesh.vertices.len(), 1058);
    let expected: Vec<[u16; 3]> = raw
        .iter()
        .map(|face| face.map(|index| if index > g { index - 1 } else { index }))
        .collect();
    assert_eq!(mesh.faces, expected);
    assert_eq!(mesh.faces.iter().flatten().max(), Some(&1057));
    for geometry in 0..2 {
        assert_eq!(
            model.meshes[at(geometry)].faces,
            file.geometries[geometry].faces.level(0).unwrap()
        );
    }
    let written = model.to_file().unwrap().write().unwrap();
    assert_eq!(
        Model::from_file(&PreFoxModel::read(&written).unwrap()).unwrap(),
        model
    );
}

#[test]
fn out_of_range_faces_numbering_vertex_count_are_repaired() {
    // CAP's faces use all 56 vertices; shifting every index past the
    // vertex count leaves 56 distinct indices, all out of range, so
    // every index is remapped to its rank and the faces come back as
    // they were.
    let original = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    let mut file = PreFoxModel::read(CAP).unwrap();
    for index in &mut file.geometries[0].faces.indices {
        *index += 56;
    }
    let model = Model::from_file(&file).unwrap();
    assert_eq!(model.meshes[0].faces, original.meshes[0].faces);
}

#[test]
fn from_file_output_passes_validate() {
    for bytes in ALL {
        Model::from_file(&PreFoxModel::read(bytes).unwrap())
            .unwrap()
            .validate()
            .unwrap();
    }
}

#[test]
fn validate_rejects_bad_mesh_references() {
    // A material past the end of the list.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].material = bad.materials.len();
    assert!(matches!(
        bad.validate(),
        Err(ModelError::BadReference {
            what: "material",
            offset: 1
        })
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::BadReference {
            what: "material",
            offset: 1
        })
    ));

    // A bone-group entry past the bone list.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].bone_group = vec![bad.bones.len()];
    assert!(matches!(
        bad.validate(),
        Err(ModelError::BadReference {
            what: "bone",
            offset: 4
        })
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::BadReference {
            what: "bone",
            offset: 4
        })
    ));

    // A face index at the vertex count, at level 0 and in a lower LOD.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].faces[0][0] = bad.meshes[0].vertices.len() as u16;
    assert!(matches!(
        bad.validate(),
        Err(ModelError::BadReference {
            what: "vertex",
            offset: 56
        })
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::BadReference {
            what: "vertex",
            offset: 56
        })
    ));

    let mut bad = Model::from_file(&PreFoxModel::read(COLLAR).unwrap()).unwrap();
    bad.meshes[0].lower_lods[0][0][0] = bad.meshes[0].vertices.len() as u16;
    assert!(matches!(
        bad.validate(),
        Err(ModelError::BadReference { what: "vertex", .. })
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::BadReference { what: "vertex", .. })
    ));
}

#[test]
fn validate_rejects_misshapen_vertices() {
    // An attribute vector one value short.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].vertices.normals.as_mut().unwrap().pop();
    assert!(matches!(
        bad.validate(),
        Err(ModelError::VertexMismatch("attribute count mismatch"))
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::VertexMismatch("attribute count mismatch"))
    ));

    // Four uv maps is the most the layout carries: valid.
    let mut model = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    let count = model.meshes[0].vertices.len();
    model.meshes[0].vertices.uvs = vec![vec![[0.0, 0.0]; count]; 4];
    model.validate().unwrap();
    model.to_file().unwrap();

    // Five uv maps.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    let count = bad.meshes[0].vertices.len();
    bad.meshes[0].vertices.uvs = vec![vec![[0.0, 0.0]; count]; 5];
    assert!(matches!(
        bad.validate(),
        Err(ModelError::InvalidVertexFormat("more than four uv maps"))
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::InvalidVertexFormat("more than four uv maps"))
    ));

    // Bone weights without bone indices.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].vertices.bone_indices = None;
    assert!(matches!(
        bad.validate(),
        Err(ModelError::InvalidVertexFormat(
            "bone weights without bone indices"
        ))
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::InvalidVertexFormat(
            "bone weights without bone indices"
        ))
    ));

    // A weight width outside 2, 3 or 4.
    let mut bad = Model::from_file(&PreFoxModel::read(CAP).unwrap()).unwrap();
    bad.meshes[0].vertices.bone_weight_width = 5;
    assert!(matches!(
        bad.validate(),
        Err(ModelError::VertexMismatch("bone weight width"))
    ));
    assert!(matches!(
        bad.to_file(),
        Err(ModelError::VertexMismatch("bone weight width"))
    ));
}
