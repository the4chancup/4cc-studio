//! Mesh splitting: `.model` meshes have hard limits on vertices, faces and
//! bone-group size that real models hit. Splitting cuts an oversized mesh
//! into component meshes under the soft limits that together contain the
//! same geometry; the split source mesh is replaced in place in
//! `Model::meshes` by its consecutive components, each carrying the
//! per-mesh extension header `Split-Mesh: N` where `N` counts the split
//! sources in the model from 1.
//!
//! A vertex may occur in several components. Two component vertices X and
//! Y describe the same source vertex iff they share the same stored byte
//! encoding (modulo bone-index differences between different bone
//! groups) *and* X and Y are both the Nth vertex of that encoding in
//! their component. To keep that convention decodable, every vertex that
//! shares a source vertex's *split key* (topological key: position, bone
//! weights and bone indices as stored) travels with it into every
//! component that takes any of them, in the same relative order; this is
//! also what keeps `vertex-loop-preservation` loops intact across a
//! split.

mod build;
mod combine;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::format::{MeshVertices, ModelError};
use crate::model::{Mesh, Model};

/// The largest bone group a `.model` mesh may reference.
pub const BONE_LIMIT_HARD: usize = 64;
/// The bone-group size a split component tries to stay below.
pub const BONE_LIMIT_SOFT: usize = 60;
/// The largest vertex count a `.model` mesh may have.
pub const VERTEX_LIMIT_HARD: usize = 65535;
/// The vertex count a split component tries to stay below.
pub const VERTEX_LIMIT_SOFT: usize = 63000;
/// The largest face count a `.model` mesh may have.
pub const FACE_LIMIT_HARD: usize = 21845;
/// The face count a split component tries to stay below.
pub const FACE_LIMIT_SOFT: usize = 20000;

/// Bone names tried first as a fragment's base bone, in order.
const PREFERRED_BASE_BONES: [&str; 5] = [
    "sk_foot_l",
    "sk_foot_r",
    "sk_hand_l",
    "sk_hand_r",
    "skf_jaw",
];

/// The extension-header key marking a split component (`Split-Mesh: N`).
pub const SPLIT_MESH_HEADER: &str = "Split-Mesh";

/// A vertex's bone mapping: `(model bone index, weight)` for every weight
/// greater than zero. Compared across components whose bone groups differ,
/// so it uses model bone indices, not bone-group slot numbers.
pub(super) type BoneMapping = Vec<(usize, f32)>;

/// Whether a mesh exceeds a hard limit (bones in its group, vertices,
/// faces).
pub fn needs_splitting(mesh: &Mesh) -> bool {
    mesh.bone_group.len() > BONE_LIMIT_HARD
        || mesh.vertices.positions.len() > VERTEX_LIMIT_HARD
        || mesh.faces.len() > FACE_LIMIT_HARD
}

/// The effective parent of every bone for splitting: `parents` as given,
/// cycles broken, and the chest/belly/hip chain inverted so `sk_chest` is
/// the root when that chain is intact.
pub fn effective_parents(model: &Model, parents: &[Option<usize>]) -> Vec<Option<usize>> {
    let mut effective = Vec::with_capacity(model.bones.len());
    for (index, &parent) in parents.iter().enumerate() {
        let mut parent = parent;
        if let Some(candidate) = parent {
            // Walking up through already-decided parents, reaching `index`
            // again would be a parent loop: cut it.
            let mut ancestor = Some(candidate);
            while let Some(bone_index) = ancestor {
                if bone_index == index {
                    parent = None;
                    break;
                }
                ancestor = effective.get(bone_index).copied().flatten();
            }
        }
        effective.push(parent);
    }

    // Invert `sk_chest -> sk_belly -> dsk_hip` when the intact chain runs
    // that way (belly's parent is the hip), so the chest is the effective
    // root. This avoids fragments that split a shirt off its torso.
    let index_of = |name: &str| model.bones.iter().position(|bone| bone.name == name);
    if let (Some(chest), Some(belly), Some(hip)) = (
        index_of("sk_chest"),
        index_of("sk_belly"),
        index_of("dsk_hip"),
    ) && effective[chest] == Some(belly)
        && effective[belly] == Some(hip)
        && effective[hip].is_none()
    {
        effective[chest] = None;
        effective[belly] = Some(chest);
        effective[hip] = Some(belly);
    }

    effective
}

/// Splits every mesh over a hard limit into component meshes under the
/// soft limits, in place in `model.meshes`. Each split mesh's components
/// sit consecutively where the source sat, each carrying the extension
/// header `Split-Mesh: N` with `N` the source's split number in the model
/// from 1. `parents` is the bone hierarchy the format does not store:
/// `parents[i]` the parent of bone `i` or `None` for a root, one entry
/// per `model.bones`; a caller without a skeleton table passes all `None`.
/// Returns whether anything was split.
pub fn encode(model: &mut Model, parents: &[Option<usize>]) -> Result<bool, ModelError> {
    if parents.len() != model.bones.len() {
        return Err(ModelError::VertexMismatch(
            "parents length does not match bone count",
        ));
    }
    let effective = effective_parents(model, parents);

    let mut components: HashMap<usize, Vec<Mesh>> = HashMap::new();
    for (index, mesh) in model.meshes.iter().enumerate() {
        if needs_splitting(mesh) {
            if !mesh.lower_lods.is_empty() {
                return Err(ModelError::InvalidModel(
                    "cannot split a mesh with LOD levels",
                ));
            }
            components.insert(index, build::split_mesh(model, mesh, &effective)?);
        }
    }
    if components.is_empty() {
        return Ok(false);
    }

    let old_meshes = std::mem::take(&mut model.meshes);
    let mut split_number = 0usize;
    for (index, mesh) in old_meshes.into_iter().enumerate() {
        match components.remove(&index) {
            None => model.meshes.push(mesh),
            Some(mut component_meshes) => {
                split_number += 1;
                let marker = format!("{SPLIT_MESH_HEADER}: {split_number}");
                for component in &mut component_meshes {
                    component.extension_headers.push(marker.clone());
                }
                model.meshes.extend(component_meshes);
            }
        }
    }
    Ok(true)
}

/// `header`'s split-group key: `Some(value)` when the header splits at its
/// first `:` into a key equal to `split-mesh` after `trim` + ASCII
/// lowercase, the value likewise trimmed and lowercased.
fn split_group_key(header: &str) -> Option<String> {
    let (key, value) = header.split_once(':')?;
    if key.trim().eq_ignore_ascii_case(SPLIT_MESH_HEADER) {
        Some(value.trim().to_lowercase())
    } else {
        None
    }
}

/// Reassembles every `Split-Mesh` group into one mesh placed where the
/// group's first component sat; meshes without the header stay. The
/// combined mesh takes the first component's `name`, `material`, `tags`,
/// `order`, `editor_data`, `bone_weight_width` and its
/// `extension_headers` minus the `Split-Mesh` header.
pub fn decode(model: &mut Model) -> Result<(), ModelError> {
    // mesh index -> the split group it is a component of.
    let group_of: Vec<Option<String>> = model
        .meshes
        .iter()
        .map(|mesh| {
            mesh.extension_headers
                .iter()
                .find_map(|header| split_group_key(header))
        })
        .collect();
    if group_of.iter().all(Option::is_none) {
        return Ok(());
    }

    // Each group's component indices, in first-appearance order.
    let mut group_keys: Vec<String> = Vec::new();
    let mut group_meshes: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, key) in group_of.iter().enumerate() {
        if let Some(key) = key {
            if !group_meshes.contains_key(key) {
                group_keys.push(key.clone());
            }
            group_meshes.entry(key.clone()).or_default().push(index);
        }
    }

    // Combine each group's components while the old mesh list is still
    // intact, then rebuild the list: components collapse to their combined
    // mesh at the first component's position.
    let old_meshes = std::mem::take(&mut model.meshes);
    let mut combined_meshes: HashMap<String, Mesh> = HashMap::new();
    for key in &group_keys {
        combined_meshes.insert(
            key.clone(),
            combine::combine(&old_meshes, &group_meshes[key])?,
        );
    }
    for (index, mesh) in old_meshes.into_iter().enumerate() {
        match &group_of[index] {
            None => model.meshes.push(mesh),
            Some(key) => {
                if group_meshes[key][0] == index {
                    model.meshes.push(combined_meshes.remove(key).ok_or(
                        ModelError::VertexMismatch("split group lost its combined mesh"),
                    )?);
                }
            }
        }
    }
    Ok(())
}

/// The stored weight of `index`'s slot `component`, or 1.0 for slot 0 and
/// 0.0 for the rest when the mesh stores bone indices only (the
/// referee-card case).
pub(super) fn weight_of(vertices: &MeshVertices, index: usize, component: usize) -> f32 {
    match &vertices.bone_weights {
        Some(weights) => weights[index][component],
        None => {
            if component == 0 {
                1.0
            } else {
                0.0
            }
        }
    }
}

/// The model bone mapping of vertex `index` of `mesh`: `(model bone index,
/// weight)` pairs for weights greater than zero. With `bone_indices` but
/// no `bone_weights`, slot 0 binds with weight 1 (the referee-card case).
pub(super) fn bone_mapping(mesh: &Mesh, index: usize) -> Result<BoneMapping, ModelError> {
    let mut mapping = Vec::new();
    if let Some(indices) = &mesh.vertices.bone_indices {
        for (component, &slot) in indices[index].iter().enumerate() {
            let weight = weight_of(&mesh.vertices, index, component);
            if weight > 0.0 {
                let slot = usize::from(slot);
                let bone = mesh
                    .bone_group
                    .get(slot)
                    .copied()
                    .ok_or(ModelError::BadReference {
                        what: "bone",
                        offset: slot as i64,
                    })?;
                mapping.push((bone, weight));
            }
        }
    }
    Ok(mapping)
}

/// Faces and loose vertex sets that contain a descendant of one bone.
#[derive(Default, Clone)]
pub(super) struct StorableItems {
    /// Face indices into the source mesh's `faces`.
    pub(super) faces: HashSet<usize>,
    /// Equipresent set indices whose vertices no face references.
    pub(super) loose: HashSet<usize>,
}

/// Copies the attributes of source vertex `source` into `vertices`,
/// remapping bone indices through `index_of` (model bone -> component bone
/// group slot; unmapped slots write 0 (a bone a zero weight never loads).
pub(super) fn push_vertex(
    vertices: &mut MeshVertices,
    source_mesh: &Mesh,
    source: usize,
    index_of: &HashMap<usize, u8>,
) -> Result<(), ModelError> {
    let source_vertices = &source_mesh.vertices;
    vertices.positions.push(source_vertices.positions[source]);
    if let (Some(target), Some(source_normals)) = (&mut vertices.normals, &source_vertices.normals)
    {
        target.push(source_normals[source]);
    }
    if let (Some(target), Some(source_tangents)) =
        (&mut vertices.tangents, &source_vertices.tangents)
    {
        target.push(source_tangents[source]);
    }
    if let (Some(target), Some(source_bitangents)) =
        (&mut vertices.bitangents, &source_vertices.bitangents)
    {
        target.push(source_bitangents[source]);
    }
    if let (Some(target), Some(source_colors)) = (&mut vertices.colors, &source_vertices.colors) {
        target.push(source_colors[source]);
    }
    for (target, source_uvs) in vertices.uvs.iter_mut().zip(&source_vertices.uvs) {
        target.push(source_uvs[source]);
    }
    if let (Some(target), Some(source_weights)) =
        (&mut vertices.bone_weights, &source_vertices.bone_weights)
    {
        target.push(source_weights[source]);
    }
    if let (Some(target), Some(source_indices)) =
        (&mut vertices.bone_indices, &source_vertices.bone_indices)
    {
        let mut remapped = [0u8; 4];
        for component in 0..4 {
            let slot = usize::from(source_indices[source][component]);
            match source_mesh.bone_group.get(slot).copied() {
                Some(bone) => {
                    if let Some(&mapped) = index_of.get(&bone) {
                        remapped[component] = mapped;
                    }
                }
                // A slot a zero weight never loads may hold anything.
                None if weight_of(source_vertices, source, component) > 0.0 => {
                    return Err(ModelError::BadReference {
                        what: "bone",
                        offset: slot as i64,
                    });
                }
                None => {}
            }
        }
        target.push(remapped);
    }
    Ok(())
}

/// An empty `MeshVertices` with `source`'s attribute layout.
pub(super) fn empty_like(source: &MeshVertices, capacity: usize) -> MeshVertices {
    MeshVertices {
        positions: Vec::with_capacity(capacity),
        normals: source
            .normals
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        tangents: source
            .tangents
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        bitangents: source
            .bitangents
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        colors: source.colors.as_ref().map(|_| Vec::with_capacity(capacity)),
        uvs: source
            .uvs
            .iter()
            .map(|_| Vec::with_capacity(capacity))
            .collect(),
        bone_indices: source
            .bone_indices
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        bone_weights: source
            .bone_weights
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        bone_weight_width: source.bone_weight_width,
    }
}
