use super::*;
use crate::format::LodRecord;
use crate::format::fixtures::*;

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
    for bytes in ALL {
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
