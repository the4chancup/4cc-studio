//! Combining a split group's components back into one mesh.

use std::collections::{BTreeSet, HashMap};

use crate::format::{FmdlError, MeshVertices};
use crate::model::{Mesh, MeshGroup};

use super::{BoneMapping, bone_mapping, empty_like, push_vertex};
use crate::ops::vertex_enc::nontopological_encoding;

/// The merge key of vertex `index`: stored bytes and bone mapping; equal
/// vertices in different bone groups still match.
fn merge_key(mesh: &Mesh, index: usize) -> Result<(Vec<u8>, BoneMapping), FmdlError> {
    let mut bytes = Vec::new();
    for component in mesh.vertices.positions[index] {
        bytes.extend(component.to_le_bytes());
    }
    bytes.extend(nontopological_encoding(&mesh.vertices, index));
    Ok((bytes, bone_mapping(mesh, index)?))
}

/// Whether `a` and `b` carry the same set of vertex attributes.
fn same_layout(a: &MeshVertices, b: &MeshVertices) -> bool {
    a.normals.is_some() == b.normals.is_some()
        && a.tangents.is_some() == b.tangents.is_some()
        && a.colors.is_some() == b.colors.is_some()
        && a.uvs.len() == b.uvs.len()
        && a.uv_high_precision == b.uv_high_precision
        && a.bone_weights.is_some() == b.bone_weights.is_some()
        && a.bone_indices.is_some() == b.bone_indices.is_some()
}

/// Reorders every attribute vector of `vertices` by `order`.
fn permute_vertices(vertices: &mut MeshVertices, order: &[usize]) {
    fn apply<T: Copy>(items: &mut Vec<T>, order: &[usize]) {
        let taken = std::mem::take(items);
        items.extend(order.iter().map(|&index| taken[index]));
    }
    apply(&mut vertices.positions, order);
    if let Some(normals) = &mut vertices.normals {
        apply(normals, order);
    }
    if let Some(tangents) = &mut vertices.tangents {
        apply(tangents, order);
    }
    if let Some(colors) = &mut vertices.colors {
        apply(colors, order);
    }
    for uvs in &mut vertices.uvs {
        apply(uvs, order);
    }
    if let Some(weights) = &mut vertices.bone_weights {
        apply(weights, order);
    }
    if let Some(indices) = &mut vertices.bone_indices {
        apply(indices, order);
    }
}

/// Combines `group`'s component meshes back into one mesh.
pub(super) fn combine(meshes: &[Mesh], group: &MeshGroup) -> Result<Mesh, FmdlError> {
    let mut components = Vec::with_capacity(group.meshes.len());
    for &mesh_index in &group.meshes {
        components.push(meshes.get(mesh_index).ok_or(FmdlError::BadReference {
            what: "mesh",
            index: mesh_index,
        })?);
    }
    let first: &Mesh = components
        .first()
        .copied()
        .ok_or(FmdlError::VertexMismatch(
            "a split-mesh group holds no meshes",
        ))?;
    for component in &components[1..] {
        if !same_layout(&component.vertices, &first.vertices) {
            return Err(FmdlError::VertexMismatch(
                "split components have different vertex layouts",
            ));
        }
    }

    // The combined bone group: every used model bone, in model bone order.
    let mut used_bones = BTreeSet::new();
    for component in &components {
        for index in 0..component.vertices.positions.len() {
            for (bone, _) in bone_mapping(component, index)? {
                used_bones.insert(bone);
            }
        }
    }
    let bone_group: Vec<usize> = used_bones.iter().copied().collect();
    let index_of: HashMap<usize, u8> = bone_group
        .iter()
        .enumerate()
        .map(|(slot, &bone)| (bone, slot as u8))
        .collect();

    let mut vertices = empty_like(&first.vertices, 0);
    let mut faces = Vec::new();
    let mut merged: HashMap<(Vec<u8>, BoneMapping), Vec<usize>> = HashMap::new();
    for component in components {
        let count = component.vertices.positions.len();
        let mut occurrences: HashMap<(Vec<u8>, BoneMapping), usize> = HashMap::new();
        let mut local = vec![0usize; count];
        for (index, slot) in local.iter_mut().enumerate() {
            let key = merge_key(component, index)?;
            let occurrence = {
                let entry = occurrences.entry(key.clone()).or_insert(0);
                let occurrence = *entry;
                *entry += 1;
                occurrence
            };
            if let Some(indices) = merged.get(&key)
                && occurrence < indices.len()
            {
                *slot = indices[occurrence];
                continue;
            }
            let merged_index = vertices.positions.len();
            push_vertex(&mut vertices, component, index, &index_of)?;
            merged.entry(key).or_default().push(merged_index);
            *slot = merged_index;
        }
        for face in &component.faces {
            let mut mapped = [0usize; 3];
            for (lane, &index) in face.iter().enumerate() {
                mapped[lane] = *local
                    .get(usize::from(index))
                    .ok_or(FmdlError::BadReference {
                        what: "face vertex",
                        index: usize::from(index),
                    })?;
            }
            faces.push(mapped);
        }
    }

    // Face indices are u16: the merged vertex order is the component
    // emission order, which can put a referenced vertex past 65535. Move
    // every referenced vertex to the front; they are deduplicates of
    // source vertices that were u16-referenced, so at most 65536 exist.
    let mut order: Vec<usize> = Vec::with_capacity(vertices.positions.len());
    let mut referenced = vec![false; vertices.positions.len()];
    for face in &faces {
        for &index in face {
            referenced[index] = true;
        }
    }
    order.extend((0..referenced.len()).filter(|&index| referenced[index]));
    order.extend((0..referenced.len()).filter(|&index| !referenced[index]));
    if let Some(&first_unreferenced) = order.get(usize::from(u16::MAX) + 1)
        && referenced[first_unreferenced]
    {
        return Err(FmdlError::VertexMismatch(
            "a combined split mesh references more than 65536 vertices",
        ));
    }
    let mut new_index = vec![0usize; order.len()];
    for (position, &vertex) in order.iter().enumerate() {
        new_index[vertex] = position;
    }
    permute_vertices(&mut vertices, &order);
    let faces: Vec<[u16; 3]> = faces
        .iter()
        .map(|face| face.map(|index| new_index[index] as u16))
        .collect();

    Ok(Mesh {
        vertices,
        faces,
        bone_group,
        material: first.material,
        alpha_flags: first.alpha_flags,
        shadow_flags: first.shadow_flags,
        has_antiblur_meshes: first.has_antiblur_meshes,
        is_antiblur_mesh: first.is_antiblur_mesh,
        custom_bounding_box: first.custom_bounding_box,
    })
}
