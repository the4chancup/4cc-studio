use super::*;
use crate::format::datum::DatumType;
use crate::format::fixtures::*;

fn round_f32s(values: &[f32], places: u32) -> Vec<f32> {
    let scale = 10f32.powi(places as i32);
    values
        .iter()
        .map(|value| (value * scale).round() / scale)
        .collect()
}

fn field_kinds(geometry: &Geometry) -> Vec<(DatumType, DatumFormat, usize)> {
    geometry
        .vertex_fields
        .iter()
        .map(|field| (field.datum_type, field.datum_format, field.count()))
        .collect()
}

#[test]
fn every_fixture_round_trips_semantically() {
    for bytes in ALL {
        let container = ModelContainer::read(bytes).unwrap();
        let model = PreFoxModel::from_container(&container).unwrap();
        let written = model.to_container().unwrap();
        // The written file parses and has the add-on's section order.
        assert_eq!(
            written.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
            [
                SectionKind::ModelBounds,
                SectionKind::BoneData,
                SectionKind::BoneNames,
                SectionKind::MaterialNames,
                SectionKind::AnnotationRecords,
                SectionKind::Cloth,
                SectionKind::MaterialCombinations,
                SectionKind::Locators,
                SectionKind::Geometry,
                SectionKind::AnnotationStrings,
                SectionKind::Meshes,
            ]
        );
        let reread_bytes = written.write();
        ModelContainer::read(&reread_bytes).unwrap();
        let reread = PreFoxModel::read(&reread_bytes).unwrap();
        assert_eq!(reread, model);
    }
    assert_eq!(PreFoxModel::read(CARDHEAD).unwrap().version, 19);
}

#[test]
fn bones_and_groups() {
    let cap = PreFoxModel::read(CAP).unwrap();
    assert_eq!(
        cap.bones
            .iter()
            .map(|bone| bone.name.as_str())
            .collect::<Vec<_>>(),
        [
            "sk_shoulder_l",
            "sk_upperarm_l",
            "dsk_deltoid_l",
            "dsk_upperarm_long_l"
        ]
    );
    assert_eq!(cap.bone_groups, [vec![0, 1, 3, 2]]);
    assert_eq!(
        round_f32s(&cap.bones[0].matrix, 6),
        [
            1.0, 0.0, 0.0, -0.105067, 0.0, 0.0, 1.0, -0.033469, 0.0, -1.0, 0.0, 1.467055
        ]
    );

    let flag = PreFoxModel::read(FLAG).unwrap();
    assert!(flag.bones.is_empty());
    assert!(flag.bone_groups.is_empty());
    assert_eq!(flag.meshes[0].bone_group, None);

    let shadow = PreFoxModel::read(SHADOW).unwrap();
    assert_eq!(shadow.bones.len(), 19);
    assert_eq!(
        shadow.bone_groups[0],
        [
            5, 4, 1, 2, 9, 8, 6, 3, 7, 10, 0, 13, 18, 14, 17, 11, 16, 12, 15
        ]
    );

    let hair = PreFoxModel::read(HAIR_HIGH).unwrap();
    assert_eq!(
        hair.bone_groups,
        [vec![4, 2, 1, 3, 10, 9, 7, 5, 8, 6, 0], vec![2, 0, 1, 3]]
    );
}

#[test]
fn geometry_fields_and_faces() {
    let cap = PreFoxModel::read(CAP).unwrap();
    let geometry = &cap.geometries[0];
    assert_eq!(
        field_kinds(geometry),
        [
            (DatumType::Position, DatumFormat::TripleFloat32, 56),
            (DatumType::Normal, DatumFormat::TripleFloat32, 56),
            (DatumType::Bitangent, DatumFormat::TripleFloat32, 56),
            (DatumType::Tangent, DatumFormat::TripleFloat32, 56),
            (DatumType::Uv0, DatumFormat::DoubleFloat32, 56),
            (DatumType::Uv1, DatumFormat::DoubleFloat32, 56),
            (DatumType::BoneIndices, DatumFormat::QuadInt8, 56),
            (DatumType::BoneWeights, DatumFormat::QuadFloat32, 56),
        ]
    );
    assert_eq!(geometry.faces.indices.len(), 234);
    assert_eq!(&geometry.faces.indices[..6], [0, 4, 1, 0, 1, 2]);
    assert!(geometry.faces.lod_ranges.is_empty());
    let first = floats_at::<3>(&geometry.vertex_fields[0].data, 0).unwrap();
    assert_eq!(round_f32s(&first, 5), [0.34969, 1.30588, -0.02537]);
    assert_eq!(
        round_f32s(&geometry.bounds.min, 4),
        [0.2507, 1.2711, -0.0383, 0.0]
    );
    assert_eq!(geometry.order, Some(0));

    let shadow = PreFoxModel::read(SHADOW).unwrap();
    assert_eq!(
        field_kinds(&shadow.geometries[0]),
        [
            (DatumType::Position, DatumFormat::TripleFloat32, 426),
            (DatumType::BoneIndices, DatumFormat::QuadInt8, 426),
            (DatumType::BoneWeights, DatumFormat::DoubleFloat32, 426),
        ]
    );
    assert_eq!(shadow.geometries[0].order, None);

    let taping = PreFoxModel::read(TAPING).unwrap();
    assert!(
        taping.geometries[0]
            .vertex_fields
            .iter()
            .any(|field| field.datum_type == DatumType::BoneWeights
                && field.datum_format == DatumFormat::TripleFloat32)
    );

    let hair = PreFoxModel::read(HAIR_HIGH).unwrap();
    assert!(field_kinds(&hair.geometries[0]).contains(&(
        DatumType::Color,
        DatumFormat::QuadFloat8,
        427
    )));
}

#[test]
fn lod_ranges_and_record() {
    let collar = PreFoxModel::read(COLLAR).unwrap();
    assert_eq!(collar.geometries[0].faces.indices.len(), 38496);
    assert_eq!(
        collar.geometries[0].faces.lod_ranges,
        [
            (0, 12240),
            (12240, 21510),
            (21510, 28353),
            (28353, 33231),
            (33231, 36510),
            (36510, 38496)
        ]
    );
    assert_eq!(collar.lod.level_count, 6);
    assert_eq!(collar.lod.parameters, [0.0625, 4.0, 0.3]);

    let card = PreFoxModel::read(CARD).unwrap();
    assert_eq!(card.lod.level_count, 0);
    assert_eq!(card.lod.parameters, [0.0625, 4.0, 0.0]);
    assert!(card.geometries[0].faces.lod_ranges.is_empty());
}

fn annotations(mesh: &Mesh) -> Vec<(usize, Option<usize>, u32, u32)> {
    mesh.annotations
        .iter()
        .map(|a| (a.string, a.record, a.unknown, a.kind))
        .collect()
}

#[test]
fn meshes_and_annotations() {
    let glasses = PreFoxModel::read(GLASSES).unwrap();
    assert_eq!(glasses.geometries.len(), 2);
    assert_eq!(glasses.meshes.len(), 2);
    assert_eq!(glasses.material_names, ["glasses_02T", "glasses_02C"]);
    assert_eq!(glasses.annotation_strings, ["glasses_02"]);
    assert_eq!(glasses.annotation_records.len(), 2);
    for record in &glasses.annotation_records {
        assert_eq!(record, &[0, 0, 0, 2, 2, 2, 0]);
    }
    let mesh = &glasses.meshes[0];
    assert_eq!(mesh.geometry, 0);
    assert_eq!(mesh.material, 0);
    assert_eq!(mesh.bone_group, Some(0));
    assert_eq!(annotations(mesh), [(0, Some(1), 7, 1), (0, Some(1), 7, 10)]);
    assert_eq!(mesh.editor_data, Some(vec![]));
    let mesh = &glasses.meshes[1];
    assert_eq!(mesh.geometry, 1);
    assert_eq!(mesh.material, 1);
    assert_eq!(mesh.bone_group, Some(1));
    assert_eq!(annotations(mesh), [(0, Some(0), 7, 1), (0, Some(0), 7, 10)]);

    let collar = PreFoxModel::read(COLLAR).unwrap();
    assert_eq!(collar.annotation_strings, ["prt", "DSpecularS", "DNormalS"]);
    assert_eq!(
        annotations(&collar.meshes[0]),
        [(0, Some(0), 7, 1), (1, Some(1), 7, 2), (2, Some(2), 7, 7)]
    );

    let head = PreFoxModel::read(HEAD_HI).unwrap();
    assert_eq!(
        annotations(&head.meshes[0]),
        [(0, Some(0), 7, 1), (1, Some(1), 7, 7)]
    );

    let cardhead = PreFoxModel::read(CARDHEAD).unwrap();
    assert_eq!(
        cardhead.annotation_strings,
        [
            "vertex-loop-preservation",
            "card",
            "Skeleton-Type: Simplified"
        ]
    );
    assert!(cardhead.annotation_records.is_empty());
    assert_eq!(
        annotations(&cardhead.meshes[0]),
        [(1, None, 7, 128), (0, None, 7, 129)]
    );
    assert_eq!(cardhead.meshes[0].editor_data, None);

    let shadow = PreFoxModel::read(SHADOW).unwrap();
    assert_eq!(shadow.meshes[0].editor_data, None);
    assert!(shadow.meshes[0].annotations.is_empty());

    let card = PreFoxModel::read(CARD).unwrap();
    assert_eq!(
        round_f32s(&card.bounds.min, 4),
        [-0.7339, 0.8666, 0.1158, 0.0]
    );
    assert_eq!(
        round_f32s(&card.bounds.max, 4),
        [-0.6078, 0.9905, 0.1463, 0.0]
    );
}

#[test]
fn editor_data_round_trips() {
    let mut model = PreFoxModel::read(CARD).unwrap();
    model.meshes[0].editor_data = Some(vec![
        EditorItem {
            kind: 1,
            unknown: 0,
            value: EditorValue::Text("TI_Util_CN_InfoNode".into()),
        },
        EditorItem {
            kind: 2,
            unknown: 0,
            value: EditorValue::Word(0),
        },
        EditorItem {
            kind: 3,
            unknown: 0,
            value: EditorValue::Word(0x13a9),
        },
    ]);
    let written = model.write().unwrap();
    assert_eq!(PreFoxModel::read(&written).unwrap(), model);
}

#[test]
fn unsupported_and_dangling_references_error() {
    // A non-empty section 8 is cloth, which the typed layer does not cover.
    let mut container = ModelContainer::read(CARD).unwrap();
    for section in &mut container.sections {
        if section.kind == SectionKind::Cloth {
            section.bytes = [12, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0].to_vec();
        }
    }
    assert_eq!(
        PreFoxModel::from_container(&container),
        Err(ModelError::Unsupported("cloth"))
    );

    // A material-combination extras word that is not `1,0,0,0`.
    let mut container = ModelContainer::read(CARD).unwrap();
    let geometry = &mut container
        .sections
        .iter_mut()
        .find(|section| section.kind == SectionKind::Geometry)
        .unwrap()
        .bytes;
    // The extras array's second entry: find the 16 bytes `1,0,0,0`.
    let needle = [1u32, 0, 0, 0]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<u8>>();
    let at = geometry
        .windows(16)
        .position(|window| window == needle.as_slice())
        .unwrap();
    geometry[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(
        PreFoxModel::from_container(&container),
        Err(ModelError::Unsupported("material combinations"))
    );

    let mut model = PreFoxModel::read(CARD).unwrap();
    model.meshes[0].material = 7;
    assert!(matches!(
        model.to_container(),
        Err(ModelError::BadReference {
            what: "material",
            ..
        })
    ));
    let mut model = PreFoxModel::read(CARD).unwrap();
    model.meshes[0].bone_group = Some(5);
    assert!(matches!(
        model.to_container(),
        Err(ModelError::BadReference {
            what: "bone group",
            ..
        })
    ));
}
