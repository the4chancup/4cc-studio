//! Combining a `Split-Mesh` group's components back into one mesh.

use std::collections::{BTreeSet, HashMap};

use crate::format::{BoundingBox, MeshVertices, ModelError};
use crate::model::Mesh;

use super::{bone_mapping, empty_like, push_vertex};
use crate::ops::vertex_enc::nontopological_encoding;

/// A component vertex's merge key: the full stored encoding plus the bone
/// mapping in model bone indices (as bytes: `f32` does not hash, so the
/// `(bytes, mapping)` pair of the fmdl version flattens into one byte
/// string), so vertices of components with different bone groups still
/// match.
fn merge_key(mesh: &Mesh, index: usize) -> Result<Vec<u8>, ModelError> {
    let mut bytes = Vec::new();
    for component in mesh.vertices.positions[index] {
        bytes.extend(component.to_le_bytes());
    }
    bytes.extend(nontopological_encoding(&mesh.vertices, index));
    for (bone, weight) in bone_mapping(mesh, index)? {
        bytes.extend((bone as u32).to_le_bytes());
        bytes.extend(weight.to_le_bytes());
    }
    Ok(bytes)
}

/// Whether two vertex lists carry the same attribute set.
fn same_layout(a: &MeshVertices, b: &MeshVertices) -> bool {
    a.normals.is_some() == b.normals.is_some()
        && a.tangents.is_some() == b.tangents.is_some()
        && a.bitangents.is_some() == b.bitangents.is_some()
        && a.colors.is_some() == b.colors.is_some()
        && a.uvs.len() == b.uvs.len()
        && a.bone_weights.is_some() == b.bone_weights.is_some()
        && a.bone_indices.is_some() == b.bone_indices.is_some()
}

/// Permutes every per-vertex list to `order`.
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
    if let Some(bitangents) = &mut vertices.bitangents {
        apply(bitangents, order);
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

/// Combines `meshes[indices]` — one `Split-Mesh` group's components — back
/// into one mesh.
pub(super) fn combine(meshes: &[Mesh], indices: &[usize]) -> Result<Mesh, ModelError> {
    let mut components = Vec::with_capacity(indices.len());
    for &mesh_index in indices {
        components.push(meshes.get(mesh_index).ok_or(ModelError::BadReference {
            what: "mesh",
            offset: mesh_index as i64,
        })?);
    }
    let first: &Mesh = components
        .first()
        .copied()
        .ok_or(ModelError::VertexMismatch(
            "a split-mesh group holds no meshes",
        ))?;
    for component in &components[1..] {
        if component.material != first.material {
            return Err(ModelError::InvalidModel(
                "split components disagree on material",
            ));
        }
        if component.vertices.bone_weight_width != first.vertices.bone_weight_width {
            return Err(ModelError::InvalidModel(
                "split components disagree on bone weight width",
            ));
        }
        if !same_layout(&component.vertices, &first.vertices) {
            return Err(ModelError::InvalidModel(
                "split components have different vertex layouts",
            ));
        }
        if !component.lower_lods.is_empty() {
            return Err(ModelError::InvalidModel(
                "a split component carries LOD levels",
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
    let mut faces: Vec<[usize; 3]> = Vec::new();
    let mut merged: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
    for component in components {
        let count = component.vertices.positions.len();
        let mut occurrences: HashMap<Vec<u8>, usize> = HashMap::new();
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
            let mut triple = [0usize; 3];
            for (slot, &index) in triple.iter_mut().zip(face) {
                *slot = local
                    .get(usize::from(index))
                    .copied()
                    .ok_or(ModelError::BadReference {
                        what: "vertex",
                        offset: i64::from(index),
                    })?;
            }
            faces.push(triple);
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
        return Err(ModelError::VertexMismatch(
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

    // The combined mesh carries the first component's metadata, minus the
    // `Split-Mesh` header: a decoded mesh is not split, and leaving it
    // would make a re-encode add a second one.
    let mut extension_headers = first.extension_headers.clone();
    if let Some(at) = extension_headers
        .iter()
        .position(|header| super::split_group_key(header).is_some())
    {
        extension_headers.remove(at);
    }
    let bounds = BoundingBox::of(&vertices.positions);
    Ok(Mesh {
        name: first.name.clone(),
        extension_headers,
        tags: first.tags.clone(),
        vertices,
        faces,
        lower_lods: Vec::new(),
        bone_group,
        material: first.material,
        bounds,
        order: first.order,
        editor_data: first.editor_data.clone(),
    })
}
