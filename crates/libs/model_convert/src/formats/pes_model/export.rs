//! The IR → `.model` + `.mtl` half: `ir_to_model`.

use ::pes_model::format::mtl;
use ::pes_model::model::Model;

use crate::ir::{CanonicalModel, validate};
use crate::loss::{self, Finding, Subject};
use crate::materials::{self, to_prefox};

use super::super::ConvertError;

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

/// `materials`'s filter value as `pes_model`'s.
fn mtl_filter(filter: materials::Filter) -> mtl::Filter {
    match filter {
        materials::Filter::Linear => mtl::Filter::Linear,
        materials::Filter::Point => mtl::Filter::Point,
        materials::Filter::Anisotropic => mtl::Filter::Anisotropic,
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

/// `[x, y, z, w]` rows truncated to xyz.
fn narrow(rows: &[[f32; 4]]) -> Vec<[f32; 3]> {
    rows.iter().map(|row| [row[0], row[1], row[2]]).collect()
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
        for name in &resolved.defaulted_samplers {
            findings.push(Finding {
                code: "sampler_settings_defaulted",
                subject: Subject::Material(index),
                detail: name.clone(),
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
