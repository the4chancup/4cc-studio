//! The pre-Fox `.model` + `.mtl` importer and exporter: `model_to_ir` decodes the mesh
//! splitting, rebuilds the parent-first bone order from the render hierarchy, and keeps the
//! `.mtl` tables verbatim on the materials; `ir_to_model` resolves each material for
//! pre-Fox, writes the inverse bind matrices back, and runs the format crate's encoders.

use std::collections::HashMap;

use ::pes_model::format::mtl;
use ::pes_model::model::Model;

use crate::affine::Affine;
use crate::ir::{
    Bone, CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Texture, Vertices, validate,
};
use crate::loss::{self, Finding, Subject};
use crate::materials::{self, PreFoxMaterial, SamplerSettings, TextureRole, family, to_prefox};
use crate::skeletons;

use super::{ConvertError, Imported, drop_out_of_group_slots};

/// A model exported to `.model` + `.mtl`: both or neither.
#[derive(Debug)]
pub struct ExportedPreFox {
    /// The `.model`.
    pub model: Model,
    /// The `.mtl`: exactly the materials the model binds, in model order.
    pub mtl: mtl::MaterialSet,
    /// The findings the export produced.
    pub findings: Vec<loss::Finding>,
}

/// `pes_model`'s filter value as `materials`'s.
fn filter(filter: mtl::Filter) -> materials::Filter {
    match filter {
        mtl::Filter::Linear => materials::Filter::Linear,
        mtl::Filter::Point => materials::Filter::Point,
        mtl::Filter::Anisotropic => materials::Filter::Anisotropic,
    }
}

/// `materials`'s filter value as `pes_model`'s.
fn mtl_filter(filter: materials::Filter) -> mtl::Filter {
    match filter {
        materials::Filter::Linear => mtl::Filter::Linear,
        materials::Filter::Point => mtl::Filter::Point,
        materials::Filter::Anisotropic => mtl::Filter::Anisotropic,
    }
}

/// `pes_model`'s address mode as `materials`'s.
fn address(address: mtl::Address) -> materials::Address {
    match address {
        mtl::Address::Wrap => materials::Address::Wrap,
        mtl::Address::Clamp => materials::Address::Clamp,
        mtl::Address::Repeat => materials::Address::Repeat,
    }
}

/// `materials`'s address mode as `pes_model`'s.
fn mtl_address(address: materials::Address) -> mtl::Address {
    match address {
        materials::Address::Wrap => mtl::Address::Wrap,
        materials::Address::Clamp => mtl::Address::Clamp,
        materials::Address::Repeat => mtl::Address::Repeat,
    }
}

/// The `Sampler`'s optional attributes as `SamplerSettings`.
fn sampler_settings(sampler: &mtl::Sampler) -> SamplerSettings {
    SamplerSettings {
        srgb: sampler.srgb,
        minfilter: sampler.minfilter.map(filter),
        magfilter: sampler.magfilter.map(filter),
        mipfilter: sampler.mipfilter.map(filter),
        uaddr: sampler.uaddr.map(address),
        vaddr: sampler.vaddr.map(address),
        waddr: sampler.waddr.map(address),
        maxaniso: sampler.maxaniso,
    }
}

/// `[x, y, z]` rows widened with `w = 1.0` (the 16→21 converter's convention).
fn widen(rows: &[[f32; 3]]) -> Vec<[f32; 4]> {
    rows.iter().map(|&[x, y, z]| [x, y, z, 1.0]).collect()
}

/// `[x, y, z, w]` rows truncated to xyz.
fn narrow(rows: &[[f32; 4]]) -> Vec<[f32; 3]> {
    rows.iter().map(|row| [row[0], row[1], row[2]]).collect()
}

/// The mesh's bone weights, synthesizing `[1, 0, 0, 0]` per vertex when it stores bone
/// indices only (the referee-card case: slot 0 binds the vertex fully). The IR keeps one
/// representation — indices and weights together — so the same binding is carried in the
/// other of the format's two spellings; no finding.
fn bone_weights(vertices: &::pes_model::format::MeshVertices) -> Option<Vec<[f32; 4]>> {
    match (&vertices.bone_indices, &vertices.bone_weights) {
        (Some(indices), None) => Some(vec![[1.0, 0.0, 0.0, 0.0]; indices.len()]),
        (_, weights) => weights.clone(),
    }
}

/// `.model` + `.mtl` → IR. Decodes the mesh splitting on a copy (the IR keeps the file's
/// vertex order), rebuilds the parent-first bone order from the render hierarchy, and
/// validates the result.
pub fn model_to_ir(model: &Model, mtl: &mtl::MaterialSet) -> Result<Imported, ConvertError> {
    let mut model = model.clone();
    ::pes_model::ops::split::decode(&mut model)?;
    let model = &model;
    let mut findings = Vec::new();

    // Each bone's render parent resolved to a bone index; a parent the model does not
    // carry leaves the bone a root (no climbing, as the legacy converters did).
    let render_parents: Vec<Option<usize>> = model
        .bones
        .iter()
        .map(|bone| {
            skeletons::render_parent(&bone.name)
                .and_then(|parent| model.bones.iter().position(|b| b.name == parent))
        })
        .collect();

    // Parent-first order: a bone listed before its parent waits on it; placing a bone
    // places its waiting children right after it, in file order.
    let mut order: Vec<usize> = Vec::with_capacity(model.bones.len());
    let mut placed = vec![false; model.bones.len()];
    let mut waiting: Vec<Vec<usize>> = vec![Vec::new(); model.bones.len()];
    let mut stack = Vec::new();
    for index in 0..model.bones.len() {
        if placed[index] {
            continue;
        }
        if let Some(parent) = render_parents[index]
            && !placed[parent]
        {
            waiting[parent].push(index);
            continue;
        }
        stack.push(index);
        while let Some(bone) = stack.pop() {
            placed[bone] = true;
            order.push(bone);
            stack.extend(waiting[bone].iter().rev());
        }
    }
    let mut new_index = vec![0usize; model.bones.len()];
    for (new, &old) in order.iter().enumerate() {
        new_index[old] = new;
    }

    let mut bones = Vec::with_capacity(model.bones.len());
    for &old in &order {
        let bone = &model.bones[old];
        bones.push(Bone {
            name: bone.name.clone(),
            parent: render_parents[old].map(|parent| new_index[parent]),
            matrix: Affine(bone.matrix)
                .inverse()
                .ok_or(ConvertError::SingularBoneMatrix(old))?,
            global_position: None,
            local_position: None,
            bounding_box: None,
        });
    }

    // Every texture the bound materials reference, deduped in first-occurrence order over
    // `model.materials` order; definitions no mesh binds are skipped.
    let mut textures: Vec<Texture> = Vec::new();
    let mut texture_index: HashMap<(&str, &str), usize> = HashMap::new();
    let mut sampler_texture: HashMap<(&str, &str), usize> = HashMap::new();
    let paths = ::pes_model::ops::paths::texture_paths(mtl);
    for name in &model.materials {
        for path in paths.iter().filter(|path| path.material == *name) {
            let key = (path.directory.as_str(), path.file_name.as_str());
            let index = match texture_index.entry(key) {
                std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(textures.len());
                    textures.push(Texture {
                        directory: path.directory.clone(),
                        file_name: path.file_name.clone(),
                    });
                    textures.len() - 1
                }
            };
            sampler_texture
                .entry((name.as_str(), path.sampler.as_str()))
                .or_insert(index);
        }
    }

    let mut materials = Vec::with_capacity(model.materials.len());
    for name in &model.materials {
        let definition = mtl
            .materials
            .iter()
            .find(|material| material.name == *name)
            .ok_or_else(|| ConvertError::MaterialUndefined(name.clone()))?;
        let inferred = family::from_prefox_shader(&definition.shader);
        if inferred.approximate {
            findings.push(Finding {
                code: "material_family_approximated",
                subject: Subject::Material(materials.len()),
                detail: name.clone(),
            });
        }
        let state = |name: &str| -> Option<u32> {
            definition.entries.iter().find_map(|entry| match entry {
                mtl::MaterialEntry::State(state) if state.name == name => Some(state.value),
                _ => None,
            })
        };
        let mut canonical: Vec<(TextureRole, usize)> = Vec::new();
        let mut native: Vec<(String, usize)> = Vec::new();
        let mut samplers = Vec::new();
        let mut states = Vec::new();
        let mut parameters = Vec::new();
        for entry in &definition.entries {
            match entry {
                mtl::MaterialEntry::Sampler(sampler) => {
                    samplers.push((sampler.name.clone(), sampler_settings(sampler)));
                    let index = *sampler_texture
                        .get(&(name.as_str(), sampler.name.as_str()))
                        .expect("texture_paths covers every sampler of a bound material");
                    match to_prefox::role_for_sampler(&sampler.name) {
                        // The first sampler a role gets is canonical; the rest are native.
                        Some(role) if !canonical.iter().any(|(seen, _)| *seen == role) => {
                            canonical.push((role, index));
                        }
                        _ => native.push((sampler.name.clone(), index)),
                    }
                }
                mtl::MaterialEntry::State(state) => {
                    states.push((state.name.clone(), state.value));
                }
                mtl::MaterialEntry::Vector(vector) => {
                    parameters.push((vector.name.clone(), vector.components.clone()));
                }
            }
        }
        materials.push(Material {
            name: name.clone(),
            family: inferred.family,
            two_sided: state("twosided").map(|value| value != 0),
            transparent: state("alphablend").map(|value| value != 0),
            antiblur: None,
            textures: canonical,
            parameters: vec![],
            fox: None,
            prefox: Some(PreFoxMaterial {
                shader: definition.shader.clone(),
                states,
                samplers,
                textures: native,
                parameters,
            }),
        });
    }

    let mut meshes = Vec::with_capacity(model.meshes.len());
    for (index, mesh) in model.meshes.iter().enumerate() {
        let vertices = &mesh.vertices;
        let synthesized = vertices.bone_indices.is_some() && vertices.bone_weights.is_none();
        let mut bone_weights = bone_weights(vertices);
        // A synthesized column writes `QuadFloat32`; a stored one keeps its width.
        let bone_weight_width = bone_weights.is_some().then_some(if synthesized {
            4
        } else {
            vertices.bone_weight_width
        });
        let dropped = match (&vertices.bone_indices, bone_weights.as_mut()) {
            (Some(indices), Some(weights)) => {
                drop_out_of_group_slots(indices, weights, mesh.bone_group.len())
            }
            _ => 0,
        };
        if dropped > 0 {
            findings.push(Finding {
                code: "bone_slot_dropped",
                subject: Subject::Mesh(index),
                detail: dropped.to_string(),
            });
        }
        // `Model::from_file` does not range-check group entries against the bone table;
        // a malformed index is an error here, not a panic on the remap.
        let mut bone_group = Vec::with_capacity(mesh.bone_group.len());
        for &index in &mesh.bone_group {
            if index >= model.bones.len() {
                return Err(ConvertError::PesModel(
                    ::pes_model::format::ModelError::BadReference {
                        what: "bone",
                        offset: index as i64,
                    },
                ));
            }
            bone_group.push(new_index[index]);
        }
        meshes.push(Mesh {
            vertices: Vertices {
                positions: vertices.positions.clone(),
                normals: vertices.normals.as_deref().map(widen),
                tangents: vertices.tangents.as_deref().map(widen),
                bitangents: vertices.bitangents.clone(),
                colors: vertices.colors.clone(),
                uvs: vertices.uvs.clone(),
                uv_high_precision: vec![false; vertices.uvs.len()],
                bone_indices: vertices.bone_indices.clone(),
                bone_weights,
                bone_weight_width,
            },
            faces: mesh.faces.iter().map(|&[a, b, c]| [a, c, b]).collect(),
            bone_group,
            material: mesh.material,
            extension_headers: mesh
                .extension_headers
                .iter()
                .filter(|header| {
                    header.as_str() != ::pes_model::ops::vertex_enc::VERTEX_LOOP_PRESERVATION
                })
                .cloned()
                .collect(),
            custom_bounding_box: None,
        });
        for (field, non_default) in [
            ("lower_lods", !mesh.lower_lods.is_empty()),
            ("tags", !mesh.tags.is_empty()),
            ("editor_data", !mesh.editor_data.is_empty()),
            ("order", mesh.order != 0),
        ] {
            if non_default {
                findings.push(Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(index),
                    detail: field.to_string(),
                });
            }
        }
    }
    if model.flags != 0 {
        findings.push(Finding {
            code: "native_field_dropped",
            subject: Subject::Model,
            detail: "flags".to_string(),
        });
    }

    let ir = CanonicalModel {
        bones,
        meshes,
        mesh_groups: model
            .meshes
            .iter()
            .enumerate()
            .map(|(index, mesh)| MeshGroup {
                name: mesh.name.clone().unwrap_or_else(|| format!("mesh_{index}")),
                parent: None,
                meshes: vec![index],
                visible: true,
            })
            .collect(),
        materials,
        textures,
        extension_headers: model.extension_headers.iter().cloned().collect(),
        source_format: SourceFormat::PreFox,
    };
    validate(&ir)?;
    Ok(Imported {
        model: ir,
        findings,
    })
}

/// IR → `.model` + `.mtl`. Validates, resolves each material for pre-Fox, writes the
/// inverse bind matrices back, and runs the format crate's encoders in the legacy
/// `saveModel` order (the vertex-loop convention, then mesh splitting).
pub fn ir_to_model(ir: &CanonicalModel) -> Result<ExportedPreFox, ConvertError> {
    validate(ir)?;
    let mut findings = Vec::new();

    let resolved: Vec<to_prefox::ResolvedPreFox> =
        ir.materials.iter().map(to_prefox::resolve).collect();
    let mut mtl_materials = Vec::with_capacity(ir.materials.len());
    for (index, (material, resolved)) in ir.materials.iter().zip(&resolved).enumerate() {
        for role in &resolved.unused_roles {
            findings.push(Finding {
                code: "material_texture_unused",
                subject: Subject::Material(index),
                detail: format!("{role:?}"),
            });
        }
        let mut entries = Vec::new();
        for (name, settings, texture) in &resolved.samplers {
            let texture = &ir.textures[*texture];
            entries.push(mtl::MaterialEntry::Sampler(mtl::Sampler {
                name: name.clone(),
                path: format!("{}{}", texture.directory, texture.file_name),
                srgb: settings.srgb,
                minfilter: settings.minfilter.map(mtl_filter),
                magfilter: settings.magfilter.map(mtl_filter),
                mipfilter: settings.mipfilter.map(mtl_filter),
                uaddr: settings.uaddr.map(mtl_address),
                vaddr: settings.vaddr.map(mtl_address),
                waddr: settings.waddr.map(mtl_address),
                maxaniso: settings.maxaniso,
            }));
        }
        entries.extend(resolved.states.iter().map(|(name, value)| {
            mtl::MaterialEntry::State(mtl::State {
                name: name.clone(),
                value: *value,
            })
        }));
        entries.extend(resolved.parameters.iter().map(|(name, components)| {
            mtl::MaterialEntry::Vector(mtl::Vector {
                name: name.clone(),
                components: components.clone(),
            })
        }));
        mtl_materials.push(mtl::Material {
            name: material.name.clone(),
            shader: resolved.shader.clone(),
            entries,
        });
    }
    let mtl = mtl::MaterialSet {
        materials: mtl_materials,
        style: mtl::MtlStyle::default(),
    };

    let mut bones = Vec::with_capacity(ir.bones.len());
    for (index, bone) in ir.bones.iter().enumerate() {
        bones.push(::pes_model::format::Bone {
            name: bone.name.clone(),
            matrix: bone
                .matrix
                .inverse()
                .ok_or(ConvertError::SingularBoneMatrix(index))?
                .0,
        });
    }

    let mut meshes = Vec::with_capacity(ir.meshes.len());
    for (index, mesh) in ir.meshes.iter().enumerate() {
        let name = ir
            .mesh_groups
            .iter()
            .find(|group| group.meshes.contains(&index))
            .map(|group| group.name.clone());
        let vertices = &mesh.vertices;
        // `.model` stores xyz only: a non-1.0 fourth component is the tangent handedness
        // (or the FMDL's 1.0 normal marker) the format cannot carry.
        for (attribute, column) in [
            ("normal_w", &vertices.normals),
            ("tangent_w", &vertices.tangents),
        ] {
            if column.iter().flatten().any(|v| v[3] != 1.0) {
                findings.push(Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(index),
                    detail: attribute.to_string(),
                });
            }
        }
        meshes.push(::pes_model::model::Mesh {
            name,
            extension_headers: mesh.extension_headers.iter().cloned().collect(),
            tags: vec![],
            vertices: ::pes_model::format::MeshVertices {
                positions: vertices.positions.clone(),
                normals: vertices.normals.as_deref().map(narrow),
                tangents: vertices.tangents.as_deref().map(narrow),
                bitangents: vertices.bitangents.clone(),
                colors: vertices.colors.clone(),
                uvs: vertices.uvs.clone(),
                bone_indices: vertices.bone_indices.clone(),
                bone_weights: vertices.bone_weights.clone(),
                bone_weight_width: vertices.bone_weight_width.unwrap_or(4),
            },
            faces: mesh.faces.iter().map(|&[a, b, c]| [a, c, b]).collect(),
            lower_lods: vec![],
            bone_group: mesh.bone_group.clone(),
            material: mesh.material,
            bounds: ::pes_model::format::BoundingBox::of(&vertices.positions),
            order: 0,
            editor_data: vec![],
        });
    }

    let positions: Vec<[f32; 3]> = ir
        .meshes
        .iter()
        .flat_map(|mesh| mesh.vertices.positions.iter().copied())
        .collect();
    let mut model = Model {
        flags: 0,
        bones,
        materials: ir
            .materials
            .iter()
            .map(|material| material.name.clone())
            .collect(),
        meshes,
        extension_headers: ir.extension_headers.iter().cloned().collect(),
        bounds: ::pes_model::format::BoundingBox::of(&positions),
        lod: ::pes_model::format::LodRecord::for_levels(0),
    };

    // The encoders in the legacy `saveModel` order: the vertex-loop convention, then
    // mesh splitting. Owners come from the IR vertex order itself (see fmdl.rs).
    let owners: Vec<Vec<usize>> = model
        .meshes
        .iter()
        .map(::pes_model::ops::vertex_enc::decode)
        .collect();
    ::pes_model::ops::vertex_enc::encode_model(&mut model, &owners)?;
    let parents: Vec<Option<usize>> = ir.bones.iter().map(|bone| bone.parent).collect();
    ::pes_model::ops::split::encode(&mut model, &parents)?;

    Ok(ExportedPreFox {
        model,
        mtl,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::fmdl;
    use crate::materials::MaterialFamily;
    use pes_version::PesVersion;

    const CAP_M: &[u8] = include_bytes!("../../tests/fixtures/konami_modD_cap.model");
    const CAP_T: &[u8] = include_bytes!("../../tests/fixtures/konami_modD_cap.mtl");
    const CARD_M: &[u8] = include_bytes!("../../tests/fixtures/konami_card.model");
    const CARD_T: &[u8] = include_bytes!("../../tests/fixtures/konami_card_red.mtl");
    const GLASSES_M: &[u8] = include_bytes!("../../tests/fixtures/konami_glasses_02.wesys.model");
    const GLASSES_T: &[u8] = include_bytes!("../../tests/fixtures/konami_accessory.mtl");
    const CARDHEAD_M: &[u8] = include_bytes!("../../tests/fixtures/cardhead_face_high.model");
    const CARDHEAD_T: &[u8] = include_bytes!("../../tests/fixtures/cardhead_materials.mtl");
    const HIGHNECK: &[u8] = include_bytes!("../../tests/fixtures/konami_highneck.fmdl");

    fn model(bytes: &[u8]) -> Model {
        Model::from_file(&::pes_model::format::PreFoxModel::read(bytes).expect("parse"))
            .expect("model")
    }

    fn set(bytes: &[u8]) -> mtl::MaterialSet {
        mtl::MaterialSet::read(bytes).expect("mtl")
    }

    /// `ir_to_model`'s output applied to `model` the same way: split decode, the
    /// indices-only weight synthesis, then the vertex-loop and split encoders with the
    /// render parents.
    fn expected(model: &Model) -> Model {
        let mut model = model.clone();
        ::pes_model::ops::split::decode(&mut model).expect("split decode");
        for mesh in &mut model.meshes {
            let synthesized =
                mesh.vertices.bone_indices.is_some() && mesh.vertices.bone_weights.is_none();
            mesh.vertices.bone_weights = bone_weights(&mesh.vertices);
            if synthesized {
                mesh.vertices.bone_weight_width = 4;
            }
        }
        let owners: Vec<Vec<usize>> = model
            .meshes
            .iter()
            .map(::pes_model::ops::vertex_enc::decode)
            .collect();
        ::pes_model::ops::vertex_enc::encode_model(&mut model, &owners).expect("vertex encode");
        let parents: Vec<Option<usize>> = model
            .bones
            .iter()
            .map(|bone| {
                skeletons::render_parent(&bone.name)
                    .and_then(|parent| model.bones.iter().position(|b| b.name == parent))
            })
            .collect();
        ::pes_model::ops::split::encode(&mut model, &parents).expect("split encode");
        model
    }

    /// One `.mtl` definition with no entries.
    fn mtl_of(name: &str, shader: &str) -> mtl::MaterialSet {
        mtl::MaterialSet {
            materials: vec![mtl::Material {
                name: name.to_string(),
                shader: shader.to_string(),
                entries: vec![],
            }],
            style: mtl::MtlStyle::default(),
        }
    }

    /// A minimal mesh over `bone_group`, weighted to its first two slots.
    fn pes_mesh(bone_group: Vec<usize>) -> ::pes_model::model::Mesh {
        ::pes_model::model::Mesh {
            name: None,
            extension_headers: vec![],
            tags: vec![],
            vertices: ::pes_model::format::MeshVertices {
                positions: vec![[0.0; 3]],
                normals: None,
                tangents: None,
                bitangents: None,
                colors: None,
                uvs: vec![],
                bone_indices: Some(vec![[0, 1, 0, 0]]),
                bone_weights: Some(vec![[0.5, 0.5, 0.0, 0.0]]),
                bone_weight_width: 4,
            },
            faces: vec![[0, 0, 0]],
            lower_lods: vec![],
            bone_group,
            material: 0,
            bounds: ::pes_model::format::BoundingBox::of(&[[0.0; 3]]),
            order: 0,
            editor_data: vec![],
        }
    }

    /// A one-mesh `Model` over `bones` for the synthetic cases.
    fn pes_model(bones: Vec<&str>, mesh: ::pes_model::model::Mesh) -> Model {
        let bounds = ::pes_model::format::BoundingBox::of(&mesh.vertices.positions);
        Model {
            flags: 0,
            bones: bones
                .into_iter()
                .map(|name| ::pes_model::format::Bone {
                    name: name.to_string(),
                    matrix: Affine::IDENTITY.0,
                })
                .collect(),
            materials: vec!["mat".to_string()],
            meshes: vec![mesh],
            extension_headers: vec![],
            bounds,
            lod: ::pes_model::format::LodRecord::for_levels(0),
        }
    }

    /// The entries of `entries` that are samplers (kind 0), states (1) or vectors (2).
    fn of_kind(entries: &[mtl::MaterialEntry], kind: u8) -> Vec<mtl::MaterialEntry> {
        entries
            .iter()
            .filter(|entry| {
                matches!(
                    (entry, kind),
                    (mtl::MaterialEntry::Sampler(_), 0)
                        | (mtl::MaterialEntry::State(_), 1)
                        | (mtl::MaterialEntry::Vector(_), 2)
                )
            })
            .cloned()
            .collect()
    }

    #[test]
    fn cap_imports_on_the_pes17_skeleton() {
        let imported = model_to_ir(&model(CAP_M), &set(CAP_T)).expect("import");
        let model = &imported.model;
        // The file order was already parent-first.
        assert_eq!(
            model
                .bones
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
        // `dsk_deltoid_l` under `sk_shoulder_l`; `dsk_upperarm_long_l`'s render parent
        // `dsk_upperarm_l` is absent, so it is a root.
        assert_eq!(model.bones[2].parent, Some(0));
        assert_eq!(model.bones[3].parent, None);
        let body = &crate::skeletons::skeletons(PesVersion::Pes17).body;
        for bone in &model.bones {
            let template = body.bone(&bone.name).expect("template bone");
            assert!(
                bone.matrix.max_component_delta(&template.matrix) < 2e-5,
                "{}",
                bone.name
            );
        }
        let material = &model.materials[0];
        assert_eq!(material.family, MaterialFamily::Shaded);
        assert_eq!(material.two_sided, None);
        assert_eq!(material.transparent, None);
        assert_eq!(
            imported.findings,
            vec![Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "tags".to_string(),
            }]
        );
    }

    #[test]
    fn pes_model_round_trip() {
        for (model_bytes, mtl_bytes, konami) in [
            (CAP_M, CAP_T, true),
            (CARD_M, CARD_T, true),
            (GLASSES_M, GLASSES_T, true),
            (CARDHEAD_M, CARDHEAD_T, false),
        ] {
            let input = model(model_bytes);
            let input_mtl = set(mtl_bytes);
            let imported = model_to_ir(&input, &input_mtl).expect("import");
            let exported = ir_to_model(&imported.model).expect("export");
            let expected = expected(&input);

            assert_eq!(
                exported
                    .model
                    .bones
                    .iter()
                    .map(|bone| bone.name.as_str())
                    .collect::<Vec<_>>(),
                expected
                    .bones
                    .iter()
                    .map(|bone| bone.name.as_str())
                    .collect::<Vec<_>>()
            );
            for (got, want) in exported.model.bones.iter().zip(&expected.bones) {
                assert!(
                    Affine(got.matrix).max_component_delta(&Affine(want.matrix)) < 1e-5,
                    "{}",
                    got.name
                );
            }
            assert_eq!(exported.model.materials, expected.materials);
            for (index, (got, want)) in exported
                .model
                .meshes
                .iter()
                .zip(&expected.meshes)
                .enumerate()
            {
                assert_eq!(got.vertices, want.vertices, "mesh {index}");
                assert_eq!(got.faces, want.faces, "mesh {index}");
                assert!(got.lower_lods.is_empty(), "mesh {index}");
                assert_eq!(got.bone_group, want.bone_group, "mesh {index}");
                assert_eq!(got.material, want.material, "mesh {index}");
                assert!(got.tags.is_empty(), "mesh {index}");
                assert_eq!(got.order, 0, "mesh {index}");
                assert!(got.editor_data.is_empty(), "mesh {index}");
                assert_eq!(
                    got.extension_headers, want.extension_headers,
                    "mesh {index}"
                );
                if konami {
                    assert_eq!(got.name, Some(format!("mesh_{index}")), "mesh {index}");
                } else {
                    assert_eq!(got.name, want.name, "mesh {index}");
                }
                assert_eq!(got.bounds, want.bounds, "mesh {index}");
            }
            assert_eq!(exported.model.extension_headers, expected.extension_headers);
            assert_eq!(exported.model.flags, 0);
            assert_eq!(
                exported.model.lod,
                ::pes_model::format::LodRecord::for_levels(0)
            );
            assert_eq!(exported.model.bounds, expected.bounds);

            // The `.mtl`: the input's definitions filtered to the bound names in model
            // order, compared per entry kind (the export regroups to samplers, states,
            // vectors — Konami's majority order).
            let want: Vec<&mtl::Material> = input
                .materials
                .iter()
                .map(|name| {
                    input_mtl
                        .materials
                        .iter()
                        .find(|material| material.name == *name)
                        .expect("bound material")
                })
                .collect();
            assert_eq!(exported.mtl.materials.len(), want.len());
            for (got, want) in exported.mtl.materials.iter().zip(want) {
                assert_eq!(got.name, want.name);
                assert_eq!(got.shader, want.shader);
                for kind in 0..3 {
                    assert_eq!(
                        of_kind(&got.entries, kind),
                        of_kind(&want.entries, kind),
                        "{} kind {kind}",
                        got.name
                    );
                }
            }
            assert_eq!(exported.mtl.style, mtl::MtlStyle::default());
        }
    }

    #[test]
    fn pes_model_round_trip_findings() {
        // cap: one Konami tag dropped.
        let imported = model_to_ir(&model(CAP_M), &set(CAP_T)).expect("import");
        assert_eq!(
            imported.findings,
            vec![Finding {
                code: "native_field_dropped",
                subject: Subject::Mesh(0),
                detail: "tags".to_string(),
            }]
        );
        assert_eq!(
            ir_to_model(&imported.model).expect("export").findings,
            Vec::<Finding>::new()
        );
        // card: nothing to report.
        let imported = model_to_ir(&model(CARD_M), &set(CARD_T)).expect("import");
        assert_eq!(imported.findings, Vec::<Finding>::new());
        assert_eq!(
            ir_to_model(&imported.model).expect("export").findings,
            Vec::<Finding>::new()
        );
        // glasses: `Accessory` is no shader rule's name; both meshes carry Konami tags.
        let imported = model_to_ir(&model(GLASSES_M), &set(GLASSES_T)).expect("import");
        assert_eq!(
            imported.findings,
            vec![
                Finding {
                    code: "material_family_approximated",
                    subject: Subject::Material(1),
                    detail: "glasses_02C".to_string(),
                },
                Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(0),
                    detail: "tags".to_string(),
                },
                Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(1),
                    detail: "tags".to_string(),
                },
            ]
        );
        assert_eq!(
            ir_to_model(&imported.model).expect("export").findings,
            Vec::<Finding>::new()
        );
        // card head: nothing to report.
        let imported = model_to_ir(&model(CARDHEAD_M), &set(CARDHEAD_T)).expect("import");
        assert_eq!(imported.findings, Vec::<Finding>::new());
        assert_eq!(
            ir_to_model(&imported.model).expect("export").findings,
            Vec::<Finding>::new()
        );
    }

    #[test]
    fn a_bone_listed_before_its_parent_moves_after_it() {
        let input = pes_model(
            vec!["dsk_forearm_l", "dsk_forearm_t_l"],
            pes_mesh(vec![0, 1]),
        );
        let imported = model_to_ir(&input, &mtl_of("mat", "Basic_C")).expect("import");
        let bones = &imported.model.bones;
        assert_eq!(bones[0].name, "dsk_forearm_t_l");
        assert_eq!(bones[0].parent, None);
        assert_eq!(bones[1].name, "dsk_forearm_l");
        assert_eq!(bones[1].parent, Some(0));
        assert_eq!(imported.model.meshes[0].bone_group, [1, 0]);
        assert_eq!(
            imported.model.meshes[0].vertices.bone_indices,
            Some(vec![[0, 1, 0, 0]])
        );
    }

    #[test]
    fn a_bound_material_without_a_definition_is_an_error() {
        let mut input = pes_model(vec!["sk_belly"], pes_mesh(vec![0]));
        input.materials = vec!["nope".to_string()];
        let empty = mtl::MaterialSet {
            materials: vec![],
            style: mtl::MtlStyle::default(),
        };
        let Err(ConvertError::MaterialUndefined(name)) = model_to_ir(&input, &empty) else {
            panic!("expected MaterialUndefined")
        };
        assert_eq!(name, "nope");
    }

    #[test]
    fn a_bone_group_past_the_bone_table_is_an_error() {
        let input = pes_model(vec!["sk_belly", "sk_chest"], pes_mesh(vec![0, 5]));
        let Err(ConvertError::PesModel(::pes_model::format::ModelError::BadReference {
            what,
            offset,
        })) = model_to_ir(&input, &mtl_of("mat", "Basic_C"))
        else {
            panic!("expected BadReference")
        };
        assert_eq!(what, "bone");
        assert_eq!(offset, 5);
    }

    #[test]
    fn dropped_fields_report_each_field() {
        let mut mesh = pes_mesh(vec![0, 1]);
        mesh.lower_lods = vec![vec![[0, 0, 0]]];
        mesh.tags = vec![(1, "part".to_string())];
        mesh.order = 3;
        let input = pes_model(vec!["sk_belly", "sk_chest"], mesh);
        let imported = model_to_ir(&input, &mtl_of("mat", "Basic_C")).expect("import");
        assert_eq!(
            imported.findings,
            vec![
                Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(0),
                    detail: "lower_lods".to_string(),
                },
                Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(0),
                    detail: "tags".to_string(),
                },
                Finding {
                    code: "native_field_dropped",
                    subject: Subject::Mesh(0),
                    detail: "order".to_string(),
                },
            ]
        );
    }

    #[test]
    fn fox_ir_exports_to_pre_fox() {
        let highneck = fmdl::fmdl_to_ir(
            &::fmdl::Model::from_file(&::fmdl::FmdlFile::read(HIGHNECK).expect("parse"))
                .expect("model"),
            None,
        )
        .expect("import");
        let exported = ir_to_model(&highneck.model).expect("export");
        assert_eq!(exported.mtl.materials.len(), 1);
        let material = &highneck.model.materials[0];
        let roles: Vec<TextureRole> = material.textures.iter().map(|(role, _)| *role).collect();
        assert_eq!(
            exported.mtl.materials[0].shader,
            to_prefox::default_shader(material.family, &roles)
        );
        assert_eq!(exported.findings, Vec::<Finding>::new());
    }

    /// A minimal consistent IR around `vertices`/`faces`: one bone, one mesh weighted
    /// to it, one `Shaded` material, one group.
    fn ir_over(vertices: Vertices, faces: Vec<[u16; 3]>) -> CanonicalModel {
        CanonicalModel {
            bones: vec![Bone {
                name: "sk_belly".to_string(),
                parent: None,
                matrix: Affine::IDENTITY,
                global_position: None,
                local_position: None,
                bounding_box: None,
            }],
            meshes: vec![Mesh {
                vertices,
                faces,
                bone_group: vec![0],
                material: 0,
                extension_headers: Default::default(),
                custom_bounding_box: None,
            }],
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
            source_format: SourceFormat::PreFox,
        }
    }

    #[test]
    fn vertex_owners_come_from_the_ir_order() {
        // Vertices 1 and 2 share position and skinning, differ only in UV: one
        // topological vertex encoded as two loops.
        let ir = ir_over(
            Vertices {
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
                uvs: vec![vec![[0.0, 0.0], [0.0, 0.0], [0.5, 0.0], [0.0, 1.0]]],
                uv_high_precision: vec![false],
                bone_indices: Some(vec![[0, 0, 0, 0]; 4]),
                bone_weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 4]),
                ..Vertices::default()
            },
            vec![[0, 1, 3], [1, 2, 3]],
        );
        let exported = ir_to_model(&ir).expect("export");
        assert_eq!(exported.model.meshes.len(), 1);
        let owner = ::pes_model::ops::vertex_enc::decode(&exported.model.meshes[0]);
        // The flag-gated `decode_model` returned identity owners [0, 1, 2, 3]; here
        // vertex 2 is the second loop of vertex 1.
        assert_eq!(owner, vec![0, 1, 1, 3]);
    }

    #[test]
    fn a_non_unit_normal_or_tangent_w_reports() {
        let vertices = |normals, tangents| Vertices {
            positions: vec![[0.0; 3]; 3],
            normals,
            tangents,
            uvs: vec![vec![[0.0, 0.0]; 3]],
            uv_high_precision: vec![false],
            bone_indices: Some(vec![[0, 0, 0, 0]; 3]),
            bone_weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
            ..Vertices::default()
        };
        let finding = |detail: &str| Finding {
            code: "native_field_dropped",
            subject: Subject::Mesh(0),
            detail: detail.to_string(),
        };
        let normals = vertices(
            Some(vec![[0.0, 0.0, 1.0, 0.0]; 3]),
            Some(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
        );
        assert_eq!(
            ir_to_model(&ir_over(normals, vec![[0, 1, 2]]))
                .expect("export")
                .findings,
            vec![finding("normal_w")]
        );
        let tangents = vertices(
            Some(vec![[0.0, 0.0, 1.0, 1.0]; 3]),
            Some(vec![[1.0, 0.0, 0.0, -1.0]; 3]),
        );
        assert_eq!(
            ir_to_model(&ir_over(tangents, vec![[0, 1, 2]]))
                .expect("export")
                .findings,
            vec![finding("tangent_w")]
        );
    }
}
